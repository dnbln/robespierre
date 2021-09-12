use robespierre::framework::standard::{Command, CommandCodeFn, StandardFramework};

fn main() {
    let _fw = StandardFramework::default()
        .configure(|c| c.prefix("!"))
        .group(|g| {
            g.name("General")
                .command(|| Command::new("ping", module::command as CommandCodeFn))
        });
}

mod module {
    use robespierre::framework::standard::{macros::command, CommandResult, FwContext};
    use robespierre_models::channels::Message;

    #[command]
    pub async fn command(_ctx: &FwContext, _msg: &Message) -> CommandResult {
        Ok(())
    }
}
