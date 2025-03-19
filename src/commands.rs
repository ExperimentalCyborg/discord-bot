use log::{debug};
use poise::CreateReply;
use poise::serenity_prelude::{Colour, CreateEmbed, CreateEmbedFooter, Mentionable};
use crate::{Context, Error};
use chrono::Local;
use clap::builder::TypedValueParser;
use rand::distr::{Distribution, Uniform};
use rand::rng;
use tokio::sync::mpsc::channel;
// Hooks ->

pub fn pre_command(ctx: Context<'_>) {
    debug!("Executing command \"{}\" ID {} for {}",
        ctx.command().qualified_name,
        ctx.id(),
        ctx.author().name
    );
}

pub fn post_command(ctx: Context<'_>) {
    debug!("Executed command \"{}\" ID {}", ctx.command().qualified_name, ctx.id());
}

pub fn command_check(ctx: Context<'_>) -> Result<bool, Error> {
    // Ok(false) // Command does not get to run
    Ok(true) // Command gets to run
}

// Commands ->

/* todo
- trackprofiles
- trackvc
- trackmodactions
 */

/// Track users joining and leaving
///
/// Events will be logged to a channel that must be specified before the feature can be enabled.
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR", subcommands("trackjoinleaves_enable", "trackjoinleaves_disable"), subcommand_required)]
pub async fn trackjoinleaves(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Enable user join/leave tracking
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR", rename = "enable")]
pub async fn trackjoinleaves_enable(
    ctx: Context<'_>,
    #[description = "Channel to log user join/leave events to"]
    channel: poise::serenity_prelude::ChannelId,
) -> Result<(), Error> {
    let guild = ctx.guild().unwrap().clone();
    let _channel = guild.channels.get(&channel);
    if _channel.is_none() {
        ctx.send(CreateReply::default()
            .content("Channel not found.".to_string())
            .ephemeral(true)
        ).await.unwrap();
        return Ok(());
    }else if _channel.unwrap().permissions_for_user(ctx.cache(), ctx.cache().current_user().id).unwrap().send_messages() == false {
        ctx.send(CreateReply::default()
            .content("I do not have permission to send messages to that channel.".to_string())
            .ephemeral(true)
        ).await.unwrap();
        return Ok(());
    }
    ctx.data().database.set_guild_value(&ctx.guild_id().unwrap(), &"config.track_joinleaves", &channel).await.unwrap();
    ctx.send(CreateReply::default()
        .content(format!("Message edit tracking enabled. Logging to {}", channel.mention()))
        .ephemeral(true)
    ).await.unwrap();
    Ok(())
}

/// Disable user join/leave tracking
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR", rename = "disable")]
pub async fn trackjoinleaves_disable(ctx: Context<'_>) -> Result<(), Error> {
    ctx.data().database.delete_guild_value(&ctx.guild_id().unwrap(), &"config.track_joinleaves").await.unwrap();
    ctx.send(CreateReply::default()
        .content("Join/leave tracking disabled.".to_string())
        .ephemeral(true)
    ).await.unwrap();
    Ok(())
}

/// Track message edits and deletions
///
/// Events will be logged to a channel that must be specified before the feature can be enabled.
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR", subcommands("trackmessageedits_enable", "trackmessageedits_disable"), subcommand_required)]
pub async fn trackmessageedits(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Enable message edit tracking
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR", rename = "enable")]
pub async fn trackmessageedits_enable(
    ctx: Context<'_>,
    #[description = "Channel to log message edits and deletions to"]
    channel: poise::serenity_prelude::ChannelId,
) -> Result<(), Error> {
    let guild = ctx.guild().unwrap().clone();
    let _channel = guild.channels.get(&channel);
    if _channel.is_none() {
        ctx.send(CreateReply::default()
            .content("Channel not found.".to_string())
            .ephemeral(true)
        ).await.unwrap();
        return Ok(());
    }else if _channel.unwrap().permissions_for_user(ctx.cache(), ctx.cache().current_user().id).unwrap().send_messages() == false {
        ctx.send(CreateReply::default()
            .content("I do not have permission to send messages to that channel.".to_string())
            .ephemeral(true)
        ).await.unwrap();
        return Ok(());
    }
    ctx.data().database.set_guild_value(&ctx.guild_id().unwrap(), &"config.track_msg_edits", &channel).await.unwrap();
    ctx.send(CreateReply::default()
        .content(format!("Message edit tracking enabled. Logging to {}", channel.mention()))
        .ephemeral(true)
    ).await.unwrap();
    Ok(())
}

/// Disable message edit tracking
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR", rename = "disable")]
pub async fn trackmessageedits_disable(ctx: Context<'_>) -> Result<(), Error> {
    ctx.data().database.delete_guild_value(&ctx.guild_id().unwrap(), &"config.track_msg_edits").await.unwrap();
    ctx.send(CreateReply::default()
        .content("Message edit tracking disabled.".to_string())
        .ephemeral(true)
    ).await.unwrap();
    Ok(())
}

/// Check if the bot is still alive
///
/// Shows timing-related real time statistics about the bot.
#[poise::command(slash_command, default_member_permissions = "SEND_MESSAGES")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let now = Local::now();
    let latency_gateway = ctx.ping().await.as_millis() / 2; // Ping is both ways
    let latency_e2e = now.timestamp_millis() - ctx.created_at().timestamp_millis();

    let colour;
    if latency_e2e <= 150 {
        colour = Colour::DARK_GREEN;
    } else if latency_e2e <= 800 {
        colour = Colour::GOLD;
    } else {
        colour = Colour::DARK_RED;
    }

    ctx.send(CreateReply::default()
        .embed(CreateEmbed::new()
            .title("⏱️ Latency statistics")
            .field("Estimated ping to user", format!("{}ms", latency_e2e * 2), false)
            .field("User to bot latency", format!("{}ms", latency_e2e), false)
            .field("Bot to gateway latency", format!("{}ms", latency_gateway), false)
            .field("Bot date/time", now.to_rfc2822(), false)
            .color(colour)
        )
    ).await?;
    Ok(())
}

/// Get detailed info about the bot
#[poise::command(slash_command, default_member_permissions = "SEND_MESSAGES")]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let now = Local::now();
    let latency_gateway = ctx.ping().await.as_millis() / 2; // Ping is both ways);
    let bot_id = ctx.cache().current_user().id.to_string();
    let avatar_url = ctx.cache().current_user().avatar_url().unwrap_or(String::from(""));

    // Note: app_* values come from Cargo.toml, edit them there!
    ctx.send(CreateReply::default()
        .embed(CreateEmbed::new()
            .color(Colour::BLITZ_BLUE)
            .thumbnail(avatar_url)
            .title("ℹ️️ About Me")
            .description(ctx.data().app_description.as_str())
            .field("Software version", ctx.data().app_version.as_str(), false)
            .field("Software author(s)", ctx.data().app_authors.as_str(), false)
            .field("Discord user ID", bot_id, false)
            .field("Bot to gateway latency", format!("{}ms", latency_gateway), false)
            .field("Boot time", ctx.data().time_started.with_timezone(&chrono::Local).to_rfc2822(), false)
            .field("Local date/time", now.to_rfc2822(), false)
            .field("Uptime", {
                let duration = chrono::Utc::now().signed_duration_since(*ctx.data().time_started);
                let days = duration.num_days();
                let hours = duration.num_hours() % 24;
                let minutes = duration.num_minutes() % 60;
                let seconds = duration.num_seconds() % 60;
                format!("{} days, {} hours, {} minutes, {} seconds", days, hours, minutes, seconds)
                },
                   false)
            .footer(CreateEmbedFooter::new("Note: The name and avatar are set by the bot owner, not the software author."))
        )
    ).await?;
    Ok(())
}

/// Show this help menu
#[poise::command(slash_command, default_member_permissions = "SEND_MESSAGES")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(ctx, command.as_deref(),
                          poise::builtins::HelpConfiguration {
                              ephemeral: true,
                              ..Default::default()
                          }).await?;
    Ok(())
}

/// Roll some dice
///
/// Rolls are calculated individually, meaning the odds are identical to rolling the given amount of physical dice.
#[poise::command(slash_command, default_member_permissions = "SEND_MESSAGES")]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "How many dice to roll. Defaults to 1"]
    rolls: Option<u8>,
    #[description = "How many sides each die has. Defaults to 6"]
    sides: Option<u8>,
    #[description = "How much to add to or subtract from the total"]
    offset: Option<i32>,
    #[description = "Describes what the roll was for"]
    purpose: Option<String>,
    #[description = "Don't show the result to anyone else"]
    hide: Option<bool>,
) -> Result<(), Error> {
    // Defaults
    let purpose = purpose.unwrap_or("".to_string());
    let hide = hide.unwrap_or(false);
    let rolls = rolls.unwrap_or(1);
    let sides = sides.unwrap_or(6);
    let offset = offset.unwrap_or(0);

    // Generate the random numbers we need
    let mut results: Vec<i32> = vec![];
    { // Ensure the compiler that rng (which isn't Send) gets dropped before the next await
        let mut generator = rng();
        let range = Uniform::try_from(1..sides).unwrap();
        for _ in 0..rolls {
            results.push(range.sample(&mut generator) as i32);
        }
    }

    // Calculate the final sum
    let sum = results.iter().sum::<i32>();
    let total = sum + offset;

    // Format the result text
    let mut text = format!("# 🎲 {}", total);
    if !purpose.is_empty() {
        text += format!(" {}", purpose).as_str();
    }

    // Build the roll explanation text
    let sign = if offset < 0 {
        ""
    }else{
        "+"
    };
    text += format!("\n{}d{}{}{} = {}", rolls, sides, sign, offset, total).as_str();

    // Send the result to the user as an embed
    ctx.send(CreateReply::default()
        .embed(CreateEmbed::new()
            .description(text)
        ).ephemeral(hide)
    ).await?;
    Ok(())
}
