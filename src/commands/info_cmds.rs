use regex_command_attr::command;

use serenity::{client::Context, model::channel::Message};

use chrono::offset::Utc;

use crate::{
    consts::DEFAULT_PREFIX,
    models::{GuildData, UserData},
    SQLPool, THEME_COLOR,
};

use std::time::{SystemTime, UNIX_EPOCH};

#[command]
#[can_blacklist(false)]
async fn ping(ctx: &Context, msg: &Message, _args: String) {
    let now = SystemTime::now();
    let since_epoch = now
        .duration_since(UNIX_EPOCH)
        .expect("Time calculated as going backwards. Very bad");

    let delta = since_epoch.as_millis() as i64 - msg.timestamp.timestamp_millis();

    let _ = msg
        .channel_id
        .say(&ctx, format!("Time taken to receive message: {}ms", delta))
        .await;
}

#[command]
#[can_blacklist(false)]
async fn help(ctx: &Context, msg: &Message, _args: String) {
    let pool = ctx
        .data
        .read()
        .await
        .get::<SQLPool>()
        .cloned()
        .expect("Could not get SQLPool from data");

    let user_data = UserData::from_user(&msg.author, &ctx, &pool).await.unwrap();
    let desc = user_data.response(&pool, "help").await;

    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(move |e| {
                e.title("Help")
                    .description(desc)
                    .footer(|f| {
                        f.text(concat!(
                            env!("CARGO_PKG_NAME"),
                            " ver ",
                            env!("CARGO_PKG_VERSION")
                        ))
                    })
                    .color(*THEME_COLOR)
            })
        })
        .await;
}

#[command]
async fn info(ctx: &Context, msg: &Message, _args: String) {
    let pool = ctx
        .data
        .read()
        .await
        .get::<SQLPool>()
        .cloned()
        .expect("Could not get SQLPool from data");

    let user_data = UserData::from_user(&msg.author, &ctx, &pool).await.unwrap();
    let guild_data = GuildData::from_guild(msg.guild(&ctx).await.unwrap(), &pool)
        .await
        .unwrap();

    let desc = user_data
        .response(&pool, "info")
        .await
        .replacen("{user}", &ctx.cache.current_user().await.name, 1)
        .replace("{default_prefix}", &*DEFAULT_PREFIX)
        .replace("{prefix}", &guild_data.prefix);

    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(move |e| {
                e.title("Info")
                    .description(desc)
                    .footer(|f| {
                        f.text(concat!(
                            env!("CARGO_PKG_NAME"),
                            " ver ",
                            env!("CARGO_PKG_VERSION")
                        ))
                    })
                    .color(*THEME_COLOR)
            })
        })
        .await;
}

#[command]
async fn donate(ctx: &Context, msg: &Message, _args: String) {
    let pool = ctx
        .data
        .read()
        .await
        .get::<SQLPool>()
        .cloned()
        .expect("Could not get SQLPool from data");

    let user_data = UserData::from_user(&msg.author, &ctx, &pool).await.unwrap();
    let desc = user_data.response(&pool, "donate").await;

    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(move |e| {
                e.title("Donate")
                    .description(desc)
                    .footer(|f| {
                        f.text(concat!(
                            env!("CARGO_PKG_NAME"),
                            " ver ",
                            env!("CARGO_PKG_VERSION")
                        ))
                    })
                    .color(*THEME_COLOR)
            })
        })
        .await;
}

#[command]
async fn dashboard(ctx: &Context, msg: &Message, _args: String) {
    let _ = msg
        .channel_id
        .send_message(ctx, |m| {
            m.embed(move |e| {
                e.title("Dashboard")
                    .description("https://reminder-bot.com/dashboard")
                    .footer(|f| {
                        f.text(concat!(
                            env!("CARGO_PKG_NAME"),
                            " ver ",
                            env!("CARGO_PKG_VERSION")
                        ))
                    })
                    .color(*THEME_COLOR)
            })
        })
        .await;
}

#[command]
async fn clock(ctx: &Context, msg: &Message, args: String) {
    let pool = ctx
        .data
        .read()
        .await
        .get::<SQLPool>()
        .cloned()
        .expect("Could not get SQLPool from data");

    let user_data = UserData::from_user(&msg.author, &ctx, &pool).await.unwrap();

    let now = Utc::now().with_timezone(&user_data.timezone());

    if args == "12" {
        let _ = msg
            .channel_id
            .say(
                &ctx,
                user_data.response(&pool, "clock/time").await.replacen(
                    "{}",
                    &now.format("%I:%M:%S %p").to_string(),
                    1,
                ),
            )
            .await;
    } else {
        let _ = msg
            .channel_id
            .say(
                &ctx,
                user_data.response(&pool, "clock/time").await.replacen(
                    "{}",
                    &now.format("%H:%M:%S").to_string(),
                    1,
                ),
            )
            .await;
    }
}
