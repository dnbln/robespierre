use std::{
    borrow::{Borrow, Cow},
    collections::HashSet,
    fmt,
    future::Future,
    pin::Pin,
    sync::Arc,
};

#[cfg(feature = "cache")]
use robespierre_cache::{Cache, HasCache};
use robespierre_models::{
    channel::{Message, MessageContent},
    id::UserId,
};

use crate::{Context, HasHttp, UserData};

use super::Framework;

#[cfg(feature = "framework-macros")]
pub mod macros {
    pub use robespierre_fw_macros::command;
}

pub mod extractors;

#[derive(Default)]
pub struct StdFwConfig {
    prefix: Cow<'static, str>,
    owners: HashSet<UserId>,
}

impl StdFwConfig {
    pub fn prefix(self, prefix: impl Into<Cow<'static, str>>) -> Self {
        Self {
            prefix: prefix.into(),
            ..self
        }
    }

    pub fn owners(self, owners: HashSet<UserId>) -> Self {
        Self { owners, ..self }
    }
}

#[derive(Default)]
pub struct StandardFramework {
    root_group: RootGroup,
    normal_message: Option<NormalMessageHandlerCode>,
    unknown_command: Option<UnknownCommandHandlerCode>,
    after: Option<AfterHandlerCode>,
    config: StdFwConfig,
}

impl StandardFramework {
    pub fn configure<F>(self, f: F) -> Self
    where
        F: FnOnce(StdFwConfig) -> StdFwConfig,
    {
        Self {
            config: f(StdFwConfig::default()),
            ..self
        }
    }

    pub fn normal_message(self, handler: impl Into<NormalMessageHandlerCode>) -> Self {
        Self {
            normal_message: Some(handler.into()),
            ..self
        }
    }

    pub fn unknown_command(self, handler: impl Into<UnknownCommandHandlerCode>) -> Self {
        Self {
            unknown_command: Some(handler.into()),
            ..self
        }
    }

    pub fn after(self, handler: impl Into<AfterHandlerCode>) -> Self {
        Self {
            after: Some(handler.into()),
            ..self
        }
    }

    pub fn group<F>(mut self, f: F) -> Self
    where
        F: for<'a> FnOnce(Group) -> Group,
    {
        let group = Group {
            name: "".into(),
            commands: vec![],
            default_invoke: None,
            subgroups: vec![],
        };
        let group = f(group);
        debug_assert!(
            group.name.as_ref() != "",
            "Name of group is \"\"; did you forget to set name of group?"
        );

        self.root_group.subgroups.push(group);
        self
    }

    async fn invoke_unknown_command(&self, ctx: &FwContext, message: &Arc<Message>) {
        if let Some(code) = self.unknown_command.as_ref() {
            code.invoke(ctx, message).await;
        }
    }

    async fn invoke_after<'a>(
        &'a self,
        ctx: &'a FwContext,
        message: &'a Arc<Message>,
        result: CommandResult,
    ) {
        if let Some(code) = self.after.as_ref() {
            code.invoke(ctx, message, result).await;
        }
    }
}

#[async_trait::async_trait]
impl Framework for StandardFramework {
    type Context = FwContext;

    async fn handle(&self, ctx: Self::Context, message: &Arc<Message>) {
        let prefix: &str = self.config.prefix.borrow();
        let message_content = match &message.content {
            MessageContent::Content(c) => c,
            MessageContent::SystemMessage(_) => return,
        };
        if let Some(command) = message_content.strip_prefix(prefix) {
            let command = self.root_group.find_command(command);

            match command {
                Some((cmd, args)) => {
                    if cmd.owners_only && !self.config.owners.contains(&message.author) {
                        self.invoke_unknown_command(&ctx, message).await;
                        return;
                    }

                    let result = cmd.code.invoke(&ctx, message, args).await;

                    self.invoke_after(&ctx, message, result).await;
                }
                None => {
                    self.invoke_unknown_command(&ctx, message).await;
                }
            }
        } else if let Some(code) = self.normal_message.as_ref() {
            code.invoke(&ctx, message).await;
        }
    }
}

#[derive(Default)]
pub struct RootGroup {
    subgroups: Vec<Group>,
}

impl RootGroup {
    pub(crate) fn find_command<'a, 'b>(
        &'a self,
        command: &'b str,
    ) -> Option<(&'a Command, &'b str)> {
        self.subgroups
            .iter()
            .find_map(|it| it.find_command(command))
    }
}

#[derive(Default)]
pub struct Group {
    name: Cow<'static, str>,
    subgroups: Vec<Group>,
    commands: Vec<Command>,
    default_invoke: Option<Command>,
}

impl Group {
    pub fn name(self, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            ..self
        }
    }

    pub fn subgroup<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Group) -> Group,
    {
        let group = f(Group::default());
        debug_assert!(
            group.name.as_ref() != "",
            "Name of group is \"\"; did you forget to set name of group?"
        );

        self.subgroups.push(group);
        self
    }

    pub fn command<F>(mut self, f: F) -> Self
    where
        F: FnOnce() -> Command,
    {
        let command = f();
        self.commands.push(command);
        self
    }

    pub fn default_command<F>(self, f: F) -> Self
    where
        F: FnOnce() -> Command,
    {
        let command = f();
        Self {
            default_invoke: Some(command),
            ..self
        }
    }
}

impl Group {
    pub(crate) fn find_command<'a, 'b>(
        &'a self,
        command: &'b str,
    ) -> Option<(&'a Command, &'b str)> {
        self.subgroups
            .iter()
            .find_map(|group| {
                let group_name: &str = group.name.borrow();
                if let Some(rest) = command.strip_prefix(group_name) {
                    if rest.trim() == "" {
                        Some((group.default_invoke.as_ref()?, ""))
                    } else if rest.starts_with(char::is_whitespace) {
                        group.find_command(rest.trim_start())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .or_else(|| {
                self.commands.iter().find_map(|c| {
                    let command_name: &str = c.name.borrow();
                    let rest = std::iter::once(command_name)
                        .chain(c.aliases.iter().map(|it| -> &str { it }))
                        .find_map(|name| command.strip_prefix(name));
                    if let Some(rest) = rest {
                        if rest.trim() == "" {
                            Some((c, ""))
                        } else if rest.starts_with(char::is_whitespace) {
                            Some((c, rest.trim_start()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
            })
            .or_else(|| Some((self.default_invoke.as_ref()?, command.trim_start())))
    }
}

#[derive(Debug)]
pub struct Command {
    name: Cow<'static, str>,
    aliases: smallvec::SmallVec<[Cow<'static, str>; 4]>,
    code: CommandCode,
    owners_only: bool,
}

impl Command {
    pub fn new(name: impl Into<Cow<'static, str>>, code: impl Into<CommandCode>) -> Self {
        Self {
            name: name.into(),
            aliases: smallvec::SmallVec::default(),
            code: code.into(),
            owners_only: false,
        }
    }

    pub fn alias(mut self, alias: impl Into<Cow<'static, str>>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    pub fn owners_only(self, owners_only: impl Into<bool>) -> Self {
        Self {
            owners_only: owners_only.into(),
            ..self
        }
    }
}

#[derive(Clone)]
pub struct FwContext {
    ctx: Context,
}

impl HasHttp for FwContext {
    fn get_http(&self) -> &robespierre_http::Http {
        self.ctx.get_http()
    }
}

#[cfg(feature = "cache")]
impl HasCache for FwContext {
    fn get_cache(&self) -> Option<&Cache> {
        self.ctx.get_cache()
    }
}

impl AsRef<Context> for FwContext {
    fn as_ref(&self) -> &Context {
        &self.ctx
    }
}

impl From<Context> for FwContext {
    fn from(ctx: Context) -> Self {
        Self { ctx }
    }
}

#[async_trait::async_trait]
impl UserData for FwContext {
    async fn data_lock_read(&self) -> tokio::sync::RwLockReadGuard<typemap::ShareMap> {
        self.ctx.data_lock_read().await
    }

    async fn data_lock_write(&self) -> tokio::sync::RwLockWriteGuard<typemap::ShareMap> {
        self.ctx.data_lock_write().await
    }
}

pub type AfterHandlerCodeFn = for<'a> fn(
    ctx: &'a FwContext,
    message: &'a Message,
    result: CommandResult,
) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub enum AfterHandlerCode {
    Binary(AfterHandlerCodeFn),
    #[cfg(feature = "interpreter")]
    Interpreted(String),
}

impl From<AfterHandlerCodeFn> for AfterHandlerCode {
    fn from(code: AfterHandlerCodeFn) -> Self {
        Self::Binary(code)
    }
}

impl AfterHandlerCode {
    pub async fn invoke<'a>(
        &'a self,
        ctx: &'a FwContext,
        message: &'a Message,
        result: CommandResult,
    ) {
        match self {
            AfterHandlerCode::Binary(f) => f(ctx, message, result).await,
            #[cfg(feature = "interpreter")]
            AfterHandlerCode::Interpreted(code) => todo!(),
        }
    }
}

pub type CommandCodeFn = for<'a> fn(
    ctx: &'a FwContext,
    message: &'a Arc<Message>,
    args: &'a str,
) -> Pin<Box<dyn Future<Output = CommandResult> + Send + 'a>>;

pub enum CommandCode {
    Binary(CommandCodeFn),
    #[cfg(feature = "interpreter")]
    Interpreted(String),
}

impl From<CommandCodeFn> for CommandCode {
    fn from(code: CommandCodeFn) -> Self {
        Self::Binary(code)
    }
}

impl fmt::Debug for CommandCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary(code) => f
                .debug_tuple("Binary")
                .field(&format_args!("{:p}", code as *const _))
                .finish(),
            #[cfg(feature = "interpreter")]
            Self::Interpreted(code) => f.debug_tuple("Interpreted").field(code).finish(),
        }
    }
}

pub type CommandError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type CommandResult<T = ()> = Result<T, CommandError>;

impl CommandCode {
    pub async fn invoke(
        &self,
        ctx: &FwContext,
        message: &Arc<Message>,
        args: &str,
    ) -> CommandResult {
        match self {
            CommandCode::Binary(f) => f(ctx, message, args).await,
            #[cfg(feature = "interpreter")]
            CommandCode::Interpreted(code) => todo!(),
        }
    }
}

pub type UnknownCommandHandlerCodeFn = for<'a> fn(
    ctx: &'a FwContext,
    message: &'a Message,
) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub enum UnknownCommandHandlerCode {
    Binary(UnknownCommandHandlerCodeFn),
    #[cfg(feature = "interpreter")]
    Interpreted(String),
}

impl UnknownCommandHandlerCode {
    pub async fn invoke(&self, ctx: &FwContext, message: &Message) {
        match self {
            Self::Binary(f) => f(ctx, message).await,
            #[cfg(feature = "interpreter")]
            Self::Interpreted(code) => todo!(),
        }
    }
}

pub type NormalMessageHandlerCodeFn = for<'a> fn(
    ctx: &'a FwContext,
    message: &'a Message,
) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
pub enum NormalMessageHandlerCode {
    Binary(NormalMessageHandlerCodeFn),
    #[cfg(feature = "interpreter")]
    Interpreted(String),
}

impl From<NormalMessageHandlerCodeFn> for NormalMessageHandlerCode {
    fn from(code: NormalMessageHandlerCodeFn) -> Self {
        Self::Binary(code)
    }
}

impl NormalMessageHandlerCode {
    pub async fn invoke(&self, ctx: &FwContext, message: &Message) {
        match self {
            Self::Binary(f) => f(ctx, message).await,
            #[cfg(feature = "interpreter")]
            Self::Interpreted(code) => todo!(),
        }
    }
}

#[cfg(test)]
mod test;
