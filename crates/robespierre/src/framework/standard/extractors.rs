use std::{borrow::Cow, future::Future, pin::Pin, sync::Arc};

use robespierre_models::{
    channel::{ChannelPermissions, Message},
    server::{Member, ServerPermissions},
    user::User,
};

use crate::model::{MessageExt, ServerIdExt};

use super::{CommandError, CommandResult, FwContext};

mod args;
pub use args::{
    Arg, Args, NeedArgValueError, NotEnoughArgs, ParseChannelError, ParseChannelIdError,
    ParseUserError, ParseUserIdError, RawArgs, Rest,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Msg {
    pub message: Arc<Message>,
    pub args: Arc<String>,
}

pub trait FromMessage: Sized {
    type Config: ExtractorConfigBuilder + Send + 'static;
    type Fut: Future<Output = CommandResult<Self>> + Send;

    fn from_message(ctx: FwContext, message: Msg, config: Self::Config) -> Self::Fut;
}

impl<T> FromMessage for Option<T>
where
    T: FromMessage,
{
    type Config = T::Config;

    type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send>>;

    fn from_message(ctx: FwContext, message: Msg, config: Self::Config) -> Self::Fut {
        Box::pin(async move {
            match T::from_message(ctx, message, config).await {
                Ok(v) => Ok::<_, CommandError>(Some(v)),
                Err(e) => {
                    tracing::debug!(
                        "Error at Option<T> as FromMessage: {}, dbg: {:?}, returning None",
                        e,
                        e
                    );
                    Ok(None)
                }
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Author(pub User);

impl FromMessage for Author {
    type Config = ();
    type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send>>;

    fn from_message(ctx: FwContext, message: Msg, _config: Self::Config) -> Self::Fut {
        Box::pin(async move {
            let fut = message.message.author(&ctx);
            Ok::<_, CommandError>(Author(fut.await?))
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("not in server")]
pub struct NotInServer;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorMember(pub Member);

impl FromMessage for AuthorMember {
    type Config = ();
    type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send>>;

    fn from_message(ctx: FwContext, message: Msg, _config: Self::Config) -> Self::Fut {
        Box::pin(async move {
            let server = message.message.server_id(&ctx).await?.ok_or(NotInServer)?;

            Ok::<_, CommandError>(AuthorMember(
                server.member(&ctx, message.message.author).await?,
            ))
        })
    }
}

pub struct RequiredPermissions<const SP: u32, const CP: u32>;

pub type RequiredServerPermissions<const SP: u32> =
    RequiredPermissions<SP, { ChannelPermissions::bits(&ChannelPermissions::empty()) }>;

pub type RequiredChannelPermissions<const CP: u32> =
    RequiredPermissions<{ ServerPermissions::bits(&ServerPermissions::empty()) }, CP>;

impl<const SP: u32, const CP: u32> FromMessage for RequiredPermissions<SP, CP> {
    type Config = ();

    type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send>>;

    fn from_message(ctx: FwContext, message: Msg, _config: Self::Config) -> Self::Fut {
        Box::pin(async move {
            super::check_perms(
                &ctx,
                &message.message,
                ServerPermissions::from_bits_truncate(SP),
                ChannelPermissions::from_bits_truncate(CP),
            )
            .await
            .map(|_| Self)
        })
    }
}

#[allow(unused_variables)]
pub trait ExtractorConfigBuilder: Sized + Default {
    fn delimiter(self, delimiter: impl Into<Cow<'static, str>>) -> Self {
        panic!("{} doesn't allow delimiters", std::any::type_name::<Self>())
    }

    fn delimiters<I, C>(self, delimiters: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Cow<'static, str>>,
    {
        panic!("{} doesn't allow delimiters", std::any::type_name::<Self>())
    }
}

impl ExtractorConfigBuilder for () {}
