use aria2_api::Client as Aria2Client;
use log::{debug, error, warn};
use std::env;
use teloxide::dptree::deps;
use teloxide::{prelude::*, update_listeners::webhooks, utils::command::BotCommands};
use tokio::sync::broadcast;

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

    #[command(description = "get aria2 version")]
    Aria2Version,

    #[command(description = "download file, params: uri")]
    Download(String),
    #[command(description = "download status, params: gid")]
    DownloadStatus(String),
}

async fn answer(msg: Message, bot: Bot, cmd: Command, aria2c: Aria2Client) -> ResponseResult<()> {
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
        Command::Aria2Version => match aria2c.get_version().await {
            Ok(version) => {
                bot.send_message(msg.chat.id, format!("{}", version))
                    .await?
            }
            Err(e) => {
                let err_msg = format!("get version err: {:?}", e);
                error!("{:?}", err_msg);
                bot.send_message(msg.chat.id, err_msg).await?
            }
        },
        Command::Download(url) => match aria2c.add_uri(&url).await {
            Ok(gid) => {
                let ok_msg = format!("gid: {}\nurl: {}", gid, url);
                bot.send_message(msg.chat.id, ok_msg).await?
            }
            Err(e) => {
                let err_msg = format!("download {} err: {:?}", url, e);
                error!("{:?}", err_msg);
                bot.send_message(msg.chat.id, err_msg).await?
            }
        },
        Command::DownloadStatus(gid) => match aria2c.tell_status(&gid).await {
            Ok(status) => {
                let ok_msg = format!("{}", status);
                bot.send_message(msg.chat.id, ok_msg).await?
            }
            Err(e) => {
                let err_msg = format!("get download status err: {:?}, gid: {}", e, gid);
                error!("{:?}", err_msg);
                bot.send_message(msg.chat.id, err_msg).await?
            }
        },
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("start");

    let bot = Bot::from_env();
    let aria2c = new_aria2c(bot.clone()).await;

    let addr = ([0, 0, 0, 0], 8080).into();
    let url = env::var("EUGENE_BOT_DOMAIN").unwrap().parse().unwrap();
    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .unwrap();

    let handler = Update::filter_message()
        .branch(dptree::entry().filter_command::<Command>().endpoint(answer));

    Dispatcher::builder(bot, handler)
        .dependencies(deps![aria2c])
        .default_handler(|upd| async move { warn!("Unhandled update: {:?}", upd) })
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error has occurred in the dispatcher"),
        )
        .await;
}

async fn new_aria2c(bot: Bot) -> Aria2Client {
    let aria2_secret = env::var("ARIA2_SECRET").unwrap();
    let ws_addr = env::var("ARIA2_WS_ADDR").unwrap();
    let chat_id = env::var("CHAT_ID").unwrap().parse::<i64>().unwrap();

    let aria2c = Aria2Client::new(&ws_addr, Some(&aria2_secret)).await;

    // 监听 notification
    let aria2c_clone = aria2c.clone();
    tokio::spawn(async move {
        debug!("start receive notification");

        let mut receiver = aria2c_clone.notification_receiver();
        loop {
            match receiver.recv().await {
                Ok(res) => {
                    if let Err(e) = bot.send_message(ChatId(chat_id), format!("{}", res)).await {
                        error!("bot send message err: {:?}", e)
                    }
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
                Err(e) => {
                    error!("receive notification err: {:?}", e)
                }
            }
        }

        debug!("stop receive notification")
    });

    aria2c
}
