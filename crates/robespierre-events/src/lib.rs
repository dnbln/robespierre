use async_tungstenite::{
    stream::Stream,
    tokio::{connect_async, TokioAdapter},
    tungstenite::Message as TungsteniteMessage,
    WebSocketStream,
};
use futures::FutureExt;
use robespierre_models::{
    channel::{Channel, ChannelField, Message, PartialChannel, PartialMessage},
    id::{ChannelId, MemberId, MessageId, RoleId, ServerId, UserId},
    server::{
        Member, MemberField, PartialMember, PartialRole, PartialServer, RoleField, Server,
        ServerField,
    },
    user::{PartialUser, RelationshipStatus, User, UserField},
};
use std::result::Result as StdResult;
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tokio_rustls::client::TlsStream;

use serde::{Deserialize, Serialize};


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


/// Any message the client can send to the server.
#[derive(Serialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(tag = "type")]
pub enum ClientToServerEvent {
    Authenticate {
        user_id: UserId,
        session_token: String,
    },

    #[serde(rename = "Authenticate")]
    AuthenticateBot {
        token: String,
    },

    BeginTyping {
        channel: ChannelId,
    },
    EndTyping {
        channel: ChannelId,
    },
    Ping {
        time: u32,
    },
}

/// Event received after authentication.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ReadyEvent {
    pub users: Vec<User>,
    pub servers: Vec<Server>,
    pub channels: Vec<Channel>,
    pub members: Vec<Member>,
}

/// Any message that the server can send to the client.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum ServerToClientEvent {
    Error {
        error: String,
    },
    Authenticated,
    Pong {
        time: u32,
    },
    Ready {
        #[serde(flatten)]
        event: ReadyEvent,
    },
    Message {
        #[serde(flatten)]
        message: Message,
    },
    MessageUpdate {
        id: MessageId,
        channel: ChannelId,
        data: PartialMessage,
    },
    MessageDelete {
        id: MessageId,
        channel: ChannelId,
    },
    ChannelCreate {
        #[serde(flatten)]
        channel: Channel,
    },
    ChannelUpdate {
        id: ChannelId,
        data: PartialChannel,
        #[serde(default)]
        clear: Option<ChannelField>,
    },
    ChannelDelete {
        id: ChannelId,
    },
    ChannelGroupJoin {
        id: ChannelId,
        user: UserId,
    },
    ChannelGroupLeave {
        id: ChannelId,
        user: UserId,
    },
    ChannelStartTyping {
        id: ChannelId,
        user: UserId,
    },
    ChannelStopTyping {
        id: ChannelId,
        user: UserId,
    },
    ChannelAck {
        id: ChannelId,
        user: UserId,
        message_id: MessageId,
    },
    ServerUpdate {
        id: ServerId,
        data: PartialServer,
        #[serde(default)]
        clear: Option<ServerField>,
    },
    ServerDelete {
        id: ServerId,
    },
    ServerMemberUpdate {
        id: MemberId,
        data: PartialMember,
        #[serde(default)]
        clear: Option<MemberField>,
    },
    ServerMemberJoin {
        id: ServerId,
        user: UserId,
    },
    ServerMemberLeave {
        id: ServerId,
        user: UserId,
    },
    ServerRoleUpdate {
        id: ServerId,
        role_id: RoleId,
        data: PartialRole,
        #[serde(default)]
        clear: Option<RoleField>,
    },
    ServerRoleDelete {
        id: ServerId,
        role_id: RoleId,
    },
    UserUpdate {
        id: UserId,
        data: PartialUser,
        #[serde(default)]
        clear: Option<UserField>,
    },
    UserRelationship {
        id: UserId,
        user: UserId,
        status: RelationshipStatus,
    },
}

struct ConnectionInternal {
    stream: WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>,
    closed: bool,
}

/// A websocket connection.
pub struct Connection(ConnectionInternal);

/// A value that can be used to authenticate on the websocket, either as a bot or as a non-bot user.
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
        }

        loop {
            // Event::FromServer = we got an event from the server, which we should pass to the handler
            // Event::ConnectionMessage = we got a message from a handler, can be something like send "BeginTyping", "EndTyping" to the ws, try to close the socket
            // Event::Tick = we didn't get any event, but we have to ping the server or it will close the connection
            let event = futures::select! {
                event = self.0.get_event().fuse() => Event::FromServer(event),
                connection_message = rx.recv().fuse() => Event::ConnectionMessage(connection_message),
                _ = int.tick().fuse() => Event::Tick,
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
                        self.start_typing(channel).await?
                    }
                    ConnectionMessage::StopTyping { channel } => self.stop_typing(channel).await?,
                    ConnectionMessage::Close => {
                        self.0.stream.close(None).await?;
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
                    let result = self.0.hb().await;
                    if let Err(err) = result {
                        tracing::error!("hb error: {}", err);
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
}

impl ConnectionInternal {
    async fn hb(&mut self) -> Result {
        tracing::debug!("sending Ping message");

        self.send_event(ClientToServerEvent::Ping { time: 0 })
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

        self.stream
            .send(TungsteniteMessage::text(serde_json::to_string(&message)?))
            .await?;

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
                tracing::debug!("Got json: {}", &json);
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
