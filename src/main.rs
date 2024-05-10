use std::env;
use teloxide::{prelude::*, update_listeners::webhooks};

#[tokio::main]
async fn main() {
    // todo: 在外部声明
    env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();
    log::info!("Starting ping-pong bot...");

    let bot = Bot::from_env();

    let addr = ([0, 0, 0, 0], 8080).into();
    let url = env::var("EUGENE_BOT_DOMAIN").unwrap().parse().unwrap();
    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook");

    teloxide::repl_with_listener(
        bot,
        |bot: Bot, msg: Message| async move {
            bot.send_message(msg.chat.id, "pong").await?;
            Ok(())
        },
        listener,
    )
        .await;
}
