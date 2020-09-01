use regex_command_attr::command;

use serenity::{
    client::Context,
    model::{
        channel::{
            Message,
        },
    },
    framework::standard::CommandResult,
};

use chrono_tz::Tz;

use chrono::offset::Utc;

use crate::{
    THEME_COLOR,
    SQLPool,
    models::UserData,
};


#[command]
#[can_blacklist(false)]
async fn help(ctx: &Context, msg: &Message, _args: String) -> CommandResult {
    let pool = ctx.data.read().await
        .get::<SQLPool>().cloned().expect("Could not get SQLPool from data");

    let user_data = UserData::from_id(&msg.author, &ctx, &pool).await.unwrap();
    let desc = user_data.response(&pool, "help").await;

    msg.channel_id.send_message(ctx, |m| m
        .embed(move |e| e
            .title("Help")
            .description(desc)
            .color(THEME_COLOR)
        )
    ).await?;

    Ok(())
}

#[command]
async fn info(ctx: &Context, msg: &Message, _args: String) -> CommandResult {
    let pool = ctx.data.read().await
        .get::<SQLPool>().cloned().expect("Could not get SQLPool from data");

    let user_data = UserData::from_id(&msg.author, &ctx, &pool).await.unwrap();
    let desc = user_data.response(&pool, "info").await;

    msg.channel_id.send_message(ctx, |m| m
        .embed(move |e| e
            .title("Info")
            .description(desc)
            .color(THEME_COLOR)
        )
    ).await?;

    Ok(())
}

#[command]
async fn donate(ctx: &Context, msg: &Message, _args: String) -> CommandResult {
    let pool = ctx.data.read().await
        .get::<SQLPool>().cloned().expect("Could not get SQLPool from data");

    let user_data = UserData::from_id(&msg.author, &ctx, &pool).await.unwrap();
    let desc = user_data.response(&pool, "donate").await;

    msg.channel_id.send_message(ctx, |m| m
        .embed(move |e| e
            .title("Donate")
            .description(desc)
            .color(THEME_COLOR)
        )
    ).await?;

    Ok(())
}

#[command]
async fn dashboard(ctx: &Context, msg: &Message, _args: String) -> CommandResult {
    msg.channel_id.send_message(ctx, |m| m
        .embed(move |e| e
            .title("Dashboard")
            .description("https://reminder-bot.com/dashboard")
            .color(THEME_COLOR)
        )
    ).await?;

    Ok(())
}

#[command]
async fn clock(ctx: &Context, msg: &Message, args: String) -> CommandResult {
    let pool = ctx.data.read().await
        .get::<SQLPool>().cloned().expect("Could not get SQLPool from data");

    let user_data = UserData::from_id(&msg.author, &ctx, &pool).await.unwrap();

    let tz: Tz = user_data.timezone.parse().unwrap();

    let now = Utc::now().with_timezone(&tz);

    if args == "12".to_string() {
        let _ = msg.channel_id.say(&ctx, user_data.response(&pool, "clock/time").await.replacen("{}", &now.format("%I:%M:%S %p").to_string(), 1)).await;
    }
    else {
        let _ = msg.channel_id.say(&ctx, user_data.response(&pool, "clock/time").await.replacen("{}", &now.format("%H:%M:%S").to_string(), 1)).await;
    }

    Ok(())
}
