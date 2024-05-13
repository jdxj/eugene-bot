use log::warn;
use std::env;
use teloxide::{prelude::*, update_listeners::webhooks, utils::command::BotCommands};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Username(username) => {
            bot.send_message(msg.chat.id, format!("Your username is @{username}."))
                .await?
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(
                msg.chat.id,
                format!("Your username is @{username} and age is {age}."),
            )
            .await?
        }
    };

    Ok(())
}

struct MyStruct {
    name: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("start");

    let bot = Bot::from_env();

    let addr = ([0, 0, 0, 0], 8080).into();
    let url = env::var("EUGENE_BOT_DOMAIN").unwrap().parse().unwrap();
    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .unwrap();

    let handler = Update::filter_message()
        .branch(dptree::entry().filter_command::<Command>().endpoint(answer));
    Dispatcher::builder(bot, handler)
        .default_handler(|upd| async move { warn!("Unhandled update: {:?}", upd) })
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error has occurred in the dispatcher"),
        )
        .await;
}
