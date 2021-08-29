use async_tungstenite::{
    stream::Stream,
    tokio::{connect_async, TokioAdapter},
    tungstenite::Message as TungsteniteMessage,
    WebSocketStream,
};
use futures::{Future, FutureExt};
use robespierre_models::{
    channel::{Channel, ChannelField, Message, PartialChannel, PartialMessage},
    id::{ChannelId, MessageId, RoleId, ServerId, UserId},
    server::{
        Member, MemberField, PartialMember, PartialRole, PartialServer, RoleField, Server,
        ServerField,
    },
    user::{PartialUser, RelationshipStatus, User, UserField},
};
use std::{result::Result as StdResult, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_rustls::client::TlsStream;

use serde::{Deserialize, Serialize};

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
        users: Vec<User>,
        servers: Vec<Server>,
        channels: Vec<Channel>,
        members: Vec<Member>,
    },
    Message {
        #[serde(flatten)]
        message: Message,
    },
    MessageUpdate {
        id: MessageId,
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
        clear: ChannelField,
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
        clear: ServerField,
    },
    ServerDelete {
        id: ServerId,
    },
    ServerMemberUpdate {
        id: ServerId,
        data: PartialMember,
        clear: MemberField,
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
        clear: RoleField,
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
    auth: Authentication,
    closed: bool,
}
pub struct Connection(ConnectionInternal);

pub enum Authentication {
    Bot {
        token: String,
    },
    User {
        user_id: UserId,
        session_token: String,
    },
}

pub trait EventHandler: Send + Sync {
    type Fut: Future<Output = ()> + Send + 'static;

    fn handle(&self, event: ServerToClientEvent) -> Self::Fut;
}

impl Connection {
    pub async fn connect(auth: Authentication) -> Result<Self> {
        tracing::debug!("Connecting to websocket");
        let (stream, _response) = connect_async("wss://ws.revolt.chat").await?;
        let mut internal = ConnectionInternal {
            stream,
            auth,
            closed: false,
        };
        internal.authenticate().await?;

        let connection = Self(internal);

        Ok(connection)
    }

    pub async fn run<H: EventHandler>(mut self, handler: H) -> Result {
        let mut int = tokio::time::interval(std::time::Duration::from_secs(15));
        loop {
            // None = we didn't get any event, but we have to ping the server or it will close the connection
            let event = futures::select! {
                event = self.0.get_event().fuse() => Some(event),
                _ = int.tick().fuse() => None,
            };

            if let Some(event) = event {
                let event = event?;
                tokio::spawn(handler.handle(event));
            } else {
                let result = self.0.hb().await;
                if let Err(err) = result {
                    tracing::error!("hb error: {}", err);
                }
            }
        }
    }
}

impl ConnectionInternal {
    pub async fn hb(&mut self) -> Result {
        tracing::debug!("sending Ping message");

        self.send_event(ClientToServerEvent::Ping { time: 0 })
            .await?;

        Ok(())
    }

    async fn authenticate(&mut self) -> Result {
        tracing::debug!("Authenticating");
        self.send_event(match &self.auth {
            Authentication::Bot { token } => ClientToServerEvent::AuthenticateBot {
                token: token.clone(),
            },
            Authentication::User {
                user_id,
                session_token,
            } => ClientToServerEvent::Authenticate {
                user_id: user_id.clone(),
                session_token: session_token.clone(),
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

    pub async fn send_event(&mut self, message: ClientToServerEvent) -> Result {
        use futures::sink::SinkExt;

        self.stream
            .send(TungsteniteMessage::text(serde_json::to_string(&message)?))
            .await?;

        Ok(())
    }

    pub async fn get_event(&mut self) -> Result<ServerToClientEvent> {
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
