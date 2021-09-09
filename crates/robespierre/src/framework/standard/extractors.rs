use std::{
    borrow::Cow,
    convert::Infallible,
    future::{ready, Future, Ready},
    pin::Pin,
    sync::Arc,
};

use robespierre_models::{
    channel::{Channel, Message},
    id::{ChannelId, IdStringDeserializeError, UserId},
    server::Member,
    user::User,
};

use crate::model::{ChannelIdExt, MessageExt, ServerIdExt, UserIdExt};

use super::{CommandError, CommandResult, FwContext};

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawArgs(pub Arc<String>);

impl FromMessage for RawArgs {
    type Config = ();
    type Fut = Ready<CommandResult<Self>>;

    fn from_message(_ctx: FwContext, message: Msg, _config: Self::Config) -> Self::Fut {
        ready(Ok(Self(Arc::clone(&message.args))))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Args<T: ArgTuple>(pub T);

impl<T: ArgTuple> FromMessage for Args<T> {
    type Config = ArgsConfig;
    type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send + 'static>>;

    fn from_message(ctx: FwContext, message: Msg, config: Self::Config) -> Self::Fut {
        Box::pin(async move {
            let fut = T::from_message(ctx, message, config);
            Ok::<_, CommandError>(Args(fut.await?))
        })
    }
}

pub trait ArgTuple: FromMessage<Config = ArgsConfig> + Sized {}

#[derive(Debug, thiserror::Error)]
#[error("need arg value error")]
pub struct NeedArgValueError;

pub trait Arg: Sized + Send {
    type Err: std::error::Error + Send + Sync + 'static;
    type Fut: Future<Output = Result<(Self, PushBack), Self::Err>> + Send + 'static;

    const TRIM: bool = true;
    const IS_REST: bool = false;

    /// used when args are over
    fn default_arg_value() -> Result<Self, NeedArgValueError> {
        Err(NeedArgValueError)
    }

    fn parse_arg(ctx: &FwContext, msg: &Msg, s: &str) -> Self::Fut;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PushBack {
    No,
    Yes,
}

// impl<T: FromStr + Send> Arg for T
// where
//     <Self as FromStr>::Err: std::error::Error + Send + Sync + 'static,
// {
//     type Err = <Self as FromStr>::Err;
//     type Fut = Ready<Result<(Self, PushBack), Self::Err>>;

//     fn parse_arg(_ctx: &FwContext, _msg: &Msg, s: &str) -> Self::Fut {
//         ready(s.parse::<T>().map(|it| (it, PushBack::No)))
//     }
// }

impl Arg for String {
    type Err = Infallible;

    type Fut = Ready<Result<(Self, PushBack), Self::Err>>;

    const TRIM: bool = false;

    fn parse_arg(_ctx: &FwContext, _msg: &Msg, s: &str) -> Self::Fut {
        ready(Ok((s.to_string(), PushBack::No)))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseUserIdError {
    #[error("the user id mention starts with `<@` but never ends with `>`")]
    DoesNotEnd,

    #[error("parsing inner id: {0}")]
    Inner(#[from] IdStringDeserializeError),
}

impl Arg for UserId {
    type Err = ParseUserIdError;

    type Fut = Ready<Result<(Self, PushBack), Self::Err>>;

    fn parse_arg(_ctx: &FwContext, _msg: &Msg, s: &str) -> Self::Fut {
        let result = if let Some(s1) = s.strip_prefix("<@") {
            if let Some(s2) = s1.strip_suffix('>') {
                s2.parse().map_err(ParseUserIdError::Inner)
            } else {
                Err(ParseUserIdError::DoesNotEnd)
            }
        } else {
            s.parse().map_err(ParseUserIdError::Inner)
        }
        .map(|it| (it, PushBack::No));

        ready(result)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseChannelIdError {
    #[error("the user id mention starts with `<#` but never ends with `>`")]
    DoesNotEnd,

    #[error("parsing inner id: {0}")]
    Inner(#[from] IdStringDeserializeError),
}

impl Arg for ChannelId {
    type Err = ParseChannelIdError;

    type Fut = Ready<Result<(Self, PushBack), Self::Err>>;

    fn parse_arg(_ctx: &FwContext, _msg: &Msg, s: &str) -> Self::Fut {
        let result = if let Some(s1) = s.strip_prefix("<#") {
            if let Some(s2) = s1.strip_suffix('>') {
                s2.parse().map_err(ParseChannelIdError::Inner)
            } else {
                Err(ParseChannelIdError::DoesNotEnd)
            }
        } else {
            s.parse().map_err(ParseChannelIdError::Inner)
        }
        .map(|it| (it, PushBack::No));

        ready(result)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseUserError {
    #[error("parse user id error: {0}")]
    ParseId(#[from] ParseUserIdError),
    #[error("other: {0}")]
    Other(#[from] crate::Error),
}

impl Arg for User {
    type Err = ParseUserError;
    #[allow(clippy::type_complexity)]
    type Fut = Pin<Box<dyn Future<Output = Result<(Self, PushBack), Self::Err>> + Send>>;

    fn parse_arg(ctx: &FwContext, msg: &Msg, s: &str) -> Self::Fut {
        let fut = UserId::parse_arg(ctx, msg, s);
        let ctx = ctx.clone();
        Box::pin(async move { Ok((fut.await?.0.user(&ctx).await?, PushBack::No)) })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseChannelError {
    #[error("parse channel id error: {0}")]
    ParseId(#[from] ParseChannelIdError),
    #[error("other: {0}")]
    Other(#[from] crate::Error),
}

impl Arg for Channel {
    type Err = ParseChannelError;
    #[allow(clippy::type_complexity)]
    type Fut = Pin<Box<dyn Future<Output = Result<(Self, PushBack), Self::Err>> + Send>>;

    fn parse_arg(ctx: &FwContext, msg: &Msg, s: &str) -> Self::Fut {
        let fut = ChannelId::parse_arg(ctx, msg, s);
        let ctx = ctx.clone();
        Box::pin(async move { Ok((fut.await?.0.channel(&ctx).await?, PushBack::No)) })
    }
}

impl<T> Arg for Option<T>
where
    T: Arg + 'static,
{
    type Err = Infallible;

    #[allow(clippy::type_complexity)]
    type Fut = Pin<Box<dyn Future<Output = Result<(Self, PushBack), Self::Err>> + Send>>;

    const TRIM: bool = <T as Arg>::TRIM;
    const IS_REST: bool = <T as Arg>::IS_REST;

    fn default_arg_value() -> Result<Self, NeedArgValueError> {
        Ok(None)
    }

    fn parse_arg(ctx: &FwContext, msg: &Msg, s: &str) -> Self::Fut {
        let fut = T::parse_arg(ctx, msg, s);

        Box::pin(async move {
            let it = fut.await;
            match it {
                Ok((v, _)) => Ok((Some(v), PushBack::No)),
                Err(_) => Ok((None, PushBack::Yes)),
            }
        })
    }
}

pub struct Rest<T: Arg + 'static>(pub T);

impl<T> Arg for Rest<T>
where
    T: Arg,
{
    type Err = <T as Arg>::Err;
    #[allow(clippy::type_complexity)]
    type Fut = Pin<Box<dyn Future<Output = Result<(Self, PushBack), Self::Err>> + Send>>;

    const TRIM: bool = <T as Arg>::TRIM;
    const IS_REST: bool = true;

    fn parse_arg(ctx: &FwContext, msg: &Msg, s: &str) -> Self::Fut {
        let fut = T::parse_arg(ctx, msg, s);
        Box::pin(async move { fut.await.map(|it| (Self(it.0), it.1)) })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("not enough args")]
pub struct NotEnoughArgs;

impl<T> FromMessage for (T,)
where
    T: Arg,
{
    type Config = ArgsConfig;
    type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send>>;

    fn from_message(ctx: FwContext, message: Msg, config: Self::Config) -> Self::Fut {
        let config = config.or_default_delimiters();

        Box::pin(async move {
            let args_lexer = ArgsLexer::new(&message.args, config);
            let mut args_lexer = ArgsLexerWrap::new(args_lexer);

            let arg = if T::IS_REST {
                args_lexer.rest().to_opt_str()
            } else {
                args_lexer.next()
            };

            let a1 = if let Some(arg) = arg {
                let arg = if T::TRIM { arg.trim() } else { arg };
                let (v, pb) = T::parse_arg(&ctx, &message, arg).await?;

                if pb == PushBack::Yes {
                    args_lexer.push_back();
                }

                v
            } else {
                T::default_arg_value()?
            };

            Ok::<_, CommandError>((a1,))
        })
    }
}
impl<T: Arg> ArgTuple for (T,) {}

macro_rules! arg_tuple_impl {
    ($($t:ident => $name:ident),* $(,)?) => {
        impl<$($t,)*> FromMessage for ($($t,)*) where $($t: Arg, )* {
            type Config = ArgsConfig;
            type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send>>;

            fn from_message(ctx: FwContext, message: Msg, config: Self::Config) -> Self::Fut {
                let config = config.or_default_delimiters();

                Box::pin(async move {
                    let args_lexer = ArgsLexer::new(&message.args, config);
                    let mut args_lexer = ArgsLexerWrap::new(args_lexer);

                    $(
                        let arg = if $t::IS_REST {
                            args_lexer.rest().to_opt_str()
                        } else {
                            args_lexer.next()
                        };

                        let $name = if let Some(arg) = arg {
                            let arg = if $t::TRIM { arg.trim() } else { arg };
                            let (v, pb) = $t::parse_arg(&ctx, &message, arg).await?;

                            if pb == PushBack::Yes {
                                args_lexer.push_back();
                            }

                            v
                        } else {
                            $t::default_arg_value()?
                        };
                    )*

                    Ok::<_, CommandError>(($($name,)*))
                })
            }
        }
        impl<$($t,)*> ArgTuple for ($($t,)*) where $($t: Arg, )* {}
    }
}

arg_tuple_impl!(T1 => a1, T2 => a2);
arg_tuple_impl!(T1 => a1, T2 => a2, T3 => a3);
arg_tuple_impl!(T1 => a1, T2 => a2, T3 => a3, T4 => a4);
arg_tuple_impl!(T1 => a1, T2 => a2, T3 => a3, T4 => a4, T5 => a5);
arg_tuple_impl!(T1 => a1, T2 => a2, T3 => a3, T4 => a4, T5 => a5, T6 => a6);
arg_tuple_impl!(T1 => a1, T2 => a2, T3 => a3, T4 => a4, T5 => a5, T6 => a6, T7 => a7);
arg_tuple_impl!(T1 => a1, T2 => a2, T3 => a3, T4 => a4, T5 => a5, T6 => a6, T7 => a7, T8 => a8);
arg_tuple_impl!(T1 => a1, T2 => a2, T3 => a3, T4 => a4, T5 => a5, T6 => a6, T7 => a7, T8 => a8, T9 => a9);
arg_tuple_impl!(T1 => a1, T2 => a2, T3 => a3, T4 => a4, T5 => a5, T6 => a6, T7 => a7, T8 => a8, T9 => a9, T10 => a10);
arg_tuple_impl!(T1 => a1, T2 => a2, T3 => a3, T4 => a4, T5 => a5, T6 => a6, T7 => a7, T8 => a8, T9 => a9, T10 => a10, T11 => a11);

#[derive(Default)]
pub struct ArgsConfig {
    pub delimiters: smallvec::SmallVec<[Cow<'static, str>; 2]>,
}

impl ArgsConfig {
    pub fn or_default_delimiters(mut self) -> Self {
        if self.delimiters.is_empty() {
            self.delimiters = smallvec::smallvec![" ".into()];
        }

        self
    }
}

struct ArgsLexer<'a> {
    args: &'a str,
    current_pos: usize,
    args_config: ArgsConfig,
    pushed_back: Option<(usize, usize)>,
    last_arg_indices: Option<(usize, usize)>,
}

impl<'a> ArgsLexer<'a> {
    fn new(args: &'a str, args_config: ArgsConfig) -> Self {
        Self {
            args: args.trim(),
            current_pos: 0,
            args_config,
            pushed_back: None,
            last_arg_indices: None,
        }
    }

    fn push_back(&mut self) {
        self.pushed_back.clone_from(&self.last_arg_indices);
    }

    fn rest(&mut self) -> Argument<'a> {
        if self.pushed_back.is_none() && self.current_pos == self.args.len() {
            return Argument::Empty;
        }

        if self.pushed_back.is_none() {
            let pos = self.current_pos;
            self.current_pos = self.args.len();
            return Argument::Simple(&self.args[pos..]);
        }

        let pushed_back = std::mem::take(&mut self.pushed_back).unwrap();
        self.current_pos = self.args.len();
        Argument::Simple(&self.args[pushed_back.0..])
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Argument<'a> {
    Simple(&'a str),
    Empty,
}

impl<'a> Argument<'a> {
    fn to_opt_str(self) -> Option<&'a str> {
        match self {
            Argument::Simple(s) => Some(s),
            Argument::Empty => None,
        }
    }
}

impl<'a> Iterator for ArgsLexer<'a> {
    type Item = Argument<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((start, end)) = self.pushed_back {
            self.pushed_back = None;
            let s = &self.args[start..end];
            if s.is_empty() {
                return Some(Argument::Empty);
            }
            return Some(Argument::Simple(s));
        }

        let s = &self.args[self.current_pos..];
        if s.is_empty() {
            return None;
        }

        if let Some(delim) = self
            .args_config
            .delimiters
            .iter()
            .find(|it| s.starts_with(it.as_ref()))
        {
            self.last_arg_indices = Some((self.current_pos, self.current_pos));
            self.current_pos += delim.as_ref().len();
            return Some(Argument::Empty);
        }

        if let Some((delim, pos)) = self
            .args_config
            .delimiters
            .iter()
            .filter_map(|it| -> Option<(&str, usize)> {
                s.find(it.as_ref()).map(|p| (it.as_ref(), p))
            })
            .min_by(|(_, a), (_, b)| a.cmp(b))
        {
            let current_pos = self.current_pos;
            self.current_pos += pos + delim.len();

            self.last_arg_indices = Some((current_pos, current_pos + pos));

            return Some(Argument::Simple(&self.args[current_pos..current_pos + pos]));
        }

        let prev_pos = self.current_pos;
        self.current_pos = self.args.len();

        self.last_arg_indices = Some((prev_pos, self.current_pos));

        return Some(Argument::Simple(&self.args[prev_pos..]));
    }
}

struct ArgsLexerWrap<'a> {
    inner: ArgsLexer<'a>,
}

impl<'a> ArgsLexerWrap<'a> {
    fn new(inner: ArgsLexer<'a>) -> Self {
        Self { inner }
    }

    fn push_back(&mut self) {
        self.inner.push_back();
    }

    fn rest(&mut self) -> Argument<'a> {
        self.inner.rest()
    }
}

impl<'a> Iterator for ArgsLexerWrap<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        for arg in &mut self.inner {
            match arg {
                Argument::Simple(s) => return Some(s),
                Argument::Empty => {}
            }
        }

        None
    }
}

impl ExtractorConfigBuilder for ArgsConfig {
    fn delimiter(mut self, delimiter: impl Into<Cow<'static, str>>) -> Self {
        self.delimiters.push(delimiter.into());
        self
    }

    fn delimiters<I, C>(mut self, delimiters: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Cow<'static, str>>,
    {
        self.delimiters
            .extend(delimiters.into_iter().map(Into::into));
        self
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

#[cfg(test)]
mod tests;
