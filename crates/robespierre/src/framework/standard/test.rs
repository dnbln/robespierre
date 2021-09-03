use crate::model::MessageExt;

use super::*;

async fn command(fw_ctx: &FwContext, message: &Message, args: &str) -> CommandResult {
    message.reply(fw_ctx, args.to_string()).await?;
    Ok(())
}

fn cmd<'a>(
    fw_ctx: &'a FwContext,
    message: &'a Message,
    args: &'a str,
) -> Pin<Box<dyn Future<Output = CommandResult> + Send + 'a>> {
    Box::pin(command(fw_ctx, message, args))
}

fn find_command<'a, 'b>(
    prefix: &str,
    fw: &'a StandardFramework,
    command: &'b str,
) -> Option<(&'a Command, &'b str)> {
    fw.root_group
        .find_command(command.strip_prefix(prefix).unwrap())
}

fn assert_cmd_is(
    prefix: &str,
    fw: &StandardFramework,
    command: &str,
    expected_command_name: &str,
    expected_args: &str,
) {
    let (cmd, args) = find_command(prefix, fw, command).expect("Command not found");

    assert_eq!(cmd.name.as_ref(), expected_command_name);
    assert_eq!(args, expected_args);
}

#[test]
fn test_find_command() {
    let framework = StandardFramework::default()
        .configure(|c| c.prefix("!"))
        .group(|g| {
            g.name("Hello")
                .command(|| Command::new("aaa", cmd as CommandCodeFn))
                .subgroup(|g| {
                    g.name("bbb")
                        .command(|| Command::new("ccc", cmd as CommandCodeFn).alias("eee"))
                        .default_command(|| Command::new("ddd", cmd as CommandCodeFn))
                })
        });
    assert_cmd_is("!", &framework, "!aaa", "aaa", "");
    assert_cmd_is("!", &framework, "!bbb ccc", "ccc", "");
    assert_cmd_is("!", &framework, "!bbb", "ddd", "");
    assert_cmd_is("!", &framework, "!bbb ddd", "ddd", "ddd");
    assert_cmd_is("!", &framework, "!bbb eee", "ccc", "");
}
