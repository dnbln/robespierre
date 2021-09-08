use async_tungstenite::{
    stream::Stream,
    tokio::{connect_async, TokioAdapter},
    tungstenite::Message as TungsteniteMessage,
    WebSocketStream,
};
use futures::FutureExt;
use robespierre_models::{
    events::{ClientToServerEvent, ServerToClientEvent},
    id::{ChannelId, UserId},
};
use std::result::Result as StdResult;
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tokio_rustls::client::TlsStream;

pub mod typing;

/// Errors that can occur while working with ws messages / events.
#[derive(Debug, thiserror::Error)]
pub enum EventsError {
    #[error("tungstenite error: {0}")]
    WsError(#[from] async_tungstenite::tungstenite::Error),

    #[error("serialization / deserialization error: {0}")]
    DeserializationError(#[from] serde_json::Error),

    #[error("error while authenticating: {0}")]
    AuthError(String),

    #[error("websocket closed")]
    Closed,
}

pub type Result<T = ()> = StdResult<T, EventsError>;

struct ConnectionInternal {
    stream: WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>,
    closed: bool,
}

/// A websocket connection.
pub struct Connection(ConnectionInternal);

/// A value that can be used to authenticate on the websocket, either as a bot or as a non-bot user.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Authentication<'a> {
    Bot {
        token: &'a str,
    },
    User {
        user_id: UserId,
        session_token: &'a str,
    },
}

#[async_trait::async_trait]
pub trait RawEventHandler: Send + Sync + Clone + 'static {
    type Context: 'static;
    async fn handle(self, ctx: Self::Context, event: ServerToClientEvent);
}

/// A message to a [`Connection`]
#[derive(Debug, Copy, Clone)]
pub enum ConnectionMessage {
    /// Tells the [`Connection`] to emit a [`ClientToServerEvent::BeginTyping`] event in the given channel.
    StartTyping { channel: ChannelId },
    /// Tells the [`Connection`] to emit a [`ClientToServerEvent::EndTyping`] event in the given channel.
    StopTyping { channel: ChannelId },
    /// Tells the [`Connection`] to close itself, and return from the loop.
    Close,
}

#[derive(Clone, Debug)]
pub struct ConnectionMessanger(UnboundedSender<ConnectionMessage>);

impl ConnectionMessanger {
    /// Sends a message to the [`Connection`], describing something it should do.
    pub fn send(&self, message: ConnectionMessage) {
        self.0
            .send(message)
            .expect("Something went terribly wrong and the receiver closed");
    }
}

/// Trait implemented on types that can be passed as a context to [`Connection::run`],
/// but not necessary for [`RawEventHandler::Context`]
pub trait Context: Sized + Clone + Send + 'static {
    /// Gives the context a messanger it can communicate to the [`Connection`] with,
    /// allowing it to send messages like "BeginTyping", "EndTyping", or tell the connection
    /// to close itself, for a clean shutdown.
    fn set_messanger(self, messanger: ConnectionMessanger) -> Self;
}

impl Connection {
    /// Connects to the websocket, and authenticates, returning the socket or an error if it failed.
    pub async fn connect<'a>(auth: impl Into<Authentication<'a>>) -> Result<Self> {
        tracing::debug!("Connecting to websocket");
        let (stream, _response) = connect_async("wss://ws.revolt.chat").await?;
        let mut internal = ConnectionInternal {
            stream,
            closed: false,
        };
        internal.authenticate(auth.into()).await?;

        let connection = Self(internal);

        Ok(connection)
    }

    /// Runs the "main loop", listening for events on the websocket and
    /// spawning tokio tasks to handle them, cloning the context, and giving
    /// it a messanger.
    ///
    /// If you intend to implement this yourself, you can
    /// use [`Connection::get_event`] to get events and [`Connection::hb`]
    /// to "heartbeat"(send a ping message to the server so it doesn't
    /// close the socket).
    pub async fn run<C, H>(mut self, ctx: C, handler: H) -> Result
    where
        C: Context,
        H: RawEventHandler<Context = C>,
    {
        let mut int = tokio::time::interval(std::time::Duration::from_secs(15));

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ConnectionMessage>();

        enum Event {
            FromServer(Result<ServerToClientEvent>),
            ConnectionMessage(Option<ConnectionMessage>),
            Tick,
            TypingManagerTick,
        }

        let mut typing_session_manager = typing::TypingSessionManager::new();

        loop {
            // Event::FromServer = we got an event from the server, which we should pass to the handler
            // Event::ConnectionMessage = we got a message from a handler, can be something like send "BeginTyping", "EndTyping" to the ws, try to close the socket
            // Event::Tick = we didn't get any event, but we have to ping the server or it will close the connection
            // Event::TypingManagerTick = we didn't get any event, but we have to send all the "BeginTyping" events to the server or it will timeout and close them.
            let event = futures::select! {
                event = self.get_event().fuse() => Event::FromServer(event),
                connection_message = rx.recv().fuse() => Event::ConnectionMessage(connection_message),
                _ = int.tick().fuse() => Event::Tick,
                _ = typing_session_manager.tick().fuse() => Event::TypingManagerTick,
            };

            match event {
                Event::FromServer(event) => {
                    let event = event?;

                    let handler = handler.clone();
                    let ctx = ctx.clone().set_messanger(ConnectionMessanger(tx.clone()));

                    let fut = handler.handle(ctx, event);
                    tokio::spawn(fut);
                }
                Event::ConnectionMessage(Some(message)) => match message {
                    ConnectionMessage::StartTyping { channel } => {
                        typing_session_manager.start_typing(channel);
                        self.start_typing(channel).await?;
                    }
                    ConnectionMessage::StopTyping { channel } => {
                        if typing_session_manager.stop_typing(channel) {
                            // was removed
                            self.stop_typing(channel).await?;
                        }
                    }
                    ConnectionMessage::Close => {
                        self.close().await?;
                        return Ok(()); // will drop self
                    }
                },
                Event::ConnectionMessage(None) => {
                    // can never happen as the tx is never moved outside of this function,
                    // only cloned, and therefore at least one sender is not dropped
                    // also, the receiver is never dropped / closed

                    // (unless ? propagates the error in which case this block shouldn't be reached)
                    unreachable!()
                }
                Event::Tick => {
                    self.hb().await?;
                }
                Event::TypingManagerTick => {
                    for session in typing_session_manager.current_sessions() {
                        self.start_typing(*session).await?;
                    }
                }
            }
        }
    }

    /// Sends a ping message to the server, so it doesn't close the connection.
    pub async fn hb(&mut self) -> Result {
        self.0.hb().await
    }

    /// Gets the next event from the server.
    pub async fn get_event(&mut self) -> Result<ServerToClientEvent> {
        self.0.get_event().await
    }

    /// Sends a [`ClientToServerEvent::BeginTyping`] event, for the given channel.
    ///
    /// Has a timeout of ~3 seconds, so if you want it to display "... is typing"
    /// for longer than that, you have to call it again.
    pub async fn start_typing(&mut self, channel: ChannelId) -> Result {
        self.0
            .send_event(ClientToServerEvent::BeginTyping { channel })
            .await
    }

    /// Sends a [`ClientToServerEvent::EndTyping`] event, for the given channel.
    pub async fn stop_typing(&mut self, channel: ChannelId) -> Result {
        self.0
            .send_event(ClientToServerEvent::EndTyping { channel })
            .await
    }

    /// Closes the websocket.
    pub async fn close(mut self) -> Result {
        self.0.close().await?;

        Ok(())
    }
}

impl ConnectionInternal {
    async fn hb(&mut self) -> Result {
        self.send_event(ClientToServerEvent::Ping { time: 0, data: (0,) })
            .await?;

        Ok(())
    }

    async fn authenticate<'a>(&mut self, auth: Authentication<'a>) -> Result {
        tracing::debug!("Authenticating");
        self.send_event(match &auth {
            Authentication::Bot { token } => ClientToServerEvent::AuthenticateBot {
                token: token.to_string(),
            },
            Authentication::User {
                user_id,
                session_token,
            } => ClientToServerEvent::Authenticate {
                user_id: user_id.clone(),
                session_token: session_token.to_string(),
            },
        })
        .await?;

        let msg = self.get_event().await?;

        match msg {
            ServerToClientEvent::Authenticated => {}
            ServerToClientEvent::Error { error } => {
                tracing::error!("Error while authenticating: {}", error);

                return Err(EventsError::AuthError(error));
            }
            msg => {
                tracing::info!("Unexpected message after auth: {:?}", msg);
            }
        }

        Ok(())
    }

    async fn send_event(&mut self, message: ClientToServerEvent) -> Result {
        use futures::sink::SinkExt;

        let json = serde_json::to_string(&message)?;

        tracing::debug!("[>] {}", &json);

        self.stream
            .send(TungsteniteMessage::text(json))
            .await?;

        Ok(())
    }

    async fn close(&mut self) -> Result {
        self.stream.close(None).await?;
        self.closed = true;

        Ok(())
    }

    async fn get_event(&mut self) -> Result<ServerToClientEvent> {
        if self.closed {
            return Err(EventsError::Closed);
        }

        use async_std::stream::StreamExt;
        let msg: TungsteniteMessage = self
            .stream
            .next()
            .await
            .expect("Last message in ws without closing")?;

        match msg {
            TungsteniteMessage::Text(json) => {
                tracing::debug!("[<] {}", &json);
                return Ok(serde_json::from_str(&json)?);
            }
            TungsteniteMessage::Binary(b) => tracing::debug!("Got binary: {:?}", &b),
            TungsteniteMessage::Ping(ping) => tracing::debug!("Got ping: {:?}", &ping),
            TungsteniteMessage::Pong(pong) => tracing::debug!("Got pong: {:?}", &pong),
            TungsteniteMessage::Close(close) => {
                tracing::debug!("Got close: {:?}", close);
                self.closed = true;

                return Err(EventsError::Closed);
            }
        };

        unimplemented!()
    }
}
