use std::{
    borrow::Cow,
    convert::Infallible,
    future::{ready, Future, Ready},
    pin::Pin,
    sync::Arc,
};

use robespierre_models::{channel::Message, server::Member, user::User};

use crate::model::{MessageExt, ServerIdExt};

use super::{CommandError, CommandResult, FwContext};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Msg {
    pub message: Arc<Message>,
    pub args: Arc<String>,
}

pub trait FromMessage: Sized {
    type Fut: Future<Output = CommandResult<Self>> + Send;

    fn from_message(ctx: FwContext, message: Msg) -> Self::Fut;
}

pub struct Author(pub User);

impl FromMessage for Author {
    type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send>>;

    fn from_message(ctx: FwContext, message: Msg) -> Self::Fut {
        Box::pin(async move {
            let fut = message.message.author(&ctx);
            Ok::<_, CommandError>(Author(fut.await?))
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("not in server")]
pub struct NotInServer;

pub struct AuthorMember(pub Member);

impl FromMessage for AuthorMember {
    type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send>>;

    fn from_message(ctx: FwContext, message: Msg) -> Self::Fut {
        Box::pin(async move {
            let server = message.message.server_id(&ctx).await?.ok_or(NotInServer)?;

            Ok::<_, CommandError>(AuthorMember(
                server.member(&ctx, message.message.author).await?,
            ))
        })
    }
}

pub struct RawArgs(pub Arc<String>);

impl FromMessage for RawArgs {
    type Fut = Ready<CommandResult<Self>>;

    fn from_message(ctx: FwContext, message: Msg) -> Self::Fut {
        ready(Ok(Self(Arc::clone(&message.args))))
    }
}

pub struct Args<T: ArgTuple>(pub T);

impl<T: ArgTuple> FromMessage for Args<T> {
    type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send + 'static>>;

    fn from_message(ctx: FwContext, message: Msg) -> Self::Fut {
        Box::pin(async move {
            let fut = T::from_message(ctx, message);
            Ok::<_, CommandError>(Args(fut.await?))
        })
    }
}

pub trait ArgTuple: FromMessage + Sized {}

pub trait Arg: Sized + Send {
    type Err: std::error::Error + Send + Sync + 'static;
    type Fut: Future<Output = Result<(Self, PushBack), Self::Err>> + Send + 'static;

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

    fn parse_arg(ctx: &FwContext, msg: &Msg, s: &str) -> Self::Fut {
        ready(Ok((s.to_string(), PushBack::No)))
    }
}

impl<T> Arg for Option<T>
where
    T: Arg + 'static,
{
    type Err = Infallible;

    type Fut = Pin<Box<dyn Future<Output = Result<(Self, PushBack), Self::Err>> + Send>>;

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

#[derive(Debug, thiserror::Error)]
#[error("not enough args")]
pub struct NotEnoughArgs;

impl<T> FromMessage for (T,)
where
    T: Arg,
{
    type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send>>;

    fn from_message(ctx: FwContext, message: Msg) -> Self::Fut {
        Box::pin(async move {
            let args_lexer = ArgsLexer::new(
                &message.args,
                ArgsConfig {
                    delimiters: smallvec::smallvec![" ".into()],
                },
            );

            let args_iter = args_lexer.filter_map(|it| match it {
                Argument::Simple(s) => Some(s),
                _ => None,
            });

            let args = args_iter.collect::<Vec<_>>();
            let mut pos = 0_usize;

            let (a1, pb) = T::parse_arg(
                &ctx,
                &message,
                if pos < args.len() { args[pos] } else { "" },
            )
            .await?;

            if pb == PushBack::No {
                pos += 1;
            }

            Ok::<_, CommandError>((a1,))
        })
    }
}
impl<T: Arg> ArgTuple for (T,) {}

macro_rules! arg_tuple_impl {
    ($($t:ident => $name:ident),* $(,)?) => {
        impl<$($t,)*> FromMessage for ($($t,)*) where $($t: Arg, )* {
            type Fut = Pin<Box<dyn Future<Output = CommandResult<Self>> + Send>>;

            fn from_message(ctx: FwContext, message: Msg) -> Self::Fut {
                Box::pin(async move {
                    let args_lexer = ArgsLexer::new(
                        &message.args,
                        ArgsConfig {
                            delimiters: smallvec::smallvec![" ".into()],
                        },
                    );

                    let args_iter = args_lexer.filter_map(|it| match it {
                        Argument::Simple(s) => Some(s),
                        _ => None,
                    });

                    let args = args_iter.collect::<Vec<_>>();
                    let mut pos = 0_usize;

                    $(
                        let ($name, pb) = $t::parse_arg(&ctx, &message, if pos < args.len() { args[pos] } else { "" },).await?;

                        if pb == PushBack::No {
                            pos += 1;
                        }
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

pub struct ArgsConfig {
    delimiters: smallvec::SmallVec<[Cow<'static, str>; 2]>,
}

struct ArgsLexer<'a> {
    args: &'a str,
    current_pos: usize,
    args_config: ArgsConfig,
}

impl<'a> ArgsLexer<'a> {
    fn new(args: &'a str, args_config: ArgsConfig) -> Self {
        Self {
            args: args.trim(),
            current_pos: 0,
            args_config,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Argument<'a> {
    Simple(&'a str),
    Empty,
}

impl<'a> Iterator for ArgsLexer<'a> {
    type Item = Argument<'a>;

    fn next(&mut self) -> Option<Self::Item> {
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

            return Some(Argument::Simple(&self.args[current_pos..current_pos + pos]));
        }

        let prev_pos = self.current_pos;
        self.current_pos = self.args.len();

        return Some(Argument::Simple(&self.args[prev_pos..]));
    }
}

#[cfg(test)]
mod tests;
