use std::path::absolute;
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

pub fn command_check(_: Context<'_>) -> Result<bool, Error> {
    // todo implement configurable bot-muted role that disables commands entirely for a user who has it
    // Ok(false) // Command does not get to run
    Ok(true) // Command gets to run
}

// Commands ->

/* todo
- trackprofiles
- trackvc
- trackmodactions
 */

/// Log various events to custom text channels
///
/// Events will be logged to a channel that must be specified before the feature can be enabled.
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR", subcommands("trackjoinleaves", "trackmessageedits"), subcommand_required)]
pub async fn track(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await.unwrap();
    Ok(())
}

/// Track users joining and leaving
///
/// Events will be logged to a channel that must be specified before the feature can be enabled.
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR", rename = "joinleave", subcommands("trackjoinleaves_enable", "trackjoinleaves_disable"), subcommand_required)]
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
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR", rename = "messageedits", subcommands("trackmessageedits_enable", "trackmessageedits_disable"), subcommand_required)]
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
    ctx.defer().await.unwrap();
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
    ctx.defer().await.unwrap();
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
    #[description = "Describes what the roll is for"]
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

    // Buy us some more time to think; for some reason some functions can be very slow
    if hide {
        ctx.defer_ephemeral().await.unwrap();
    }else{
        ctx.defer().await.unwrap();
    }

    // Generate the random numbers we need
    let mut results: Vec<i32> = vec![];
    { // Ensure the compiler that rng (which isn't Send) gets dropped before the next await
        let mut generator = rng();
        let range = Uniform::try_from(1..=sides).unwrap();
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

/// Pick a random number
#[poise::command(slash_command, default_member_permissions = "SEND_MESSAGES")]
pub async fn number(
    ctx: Context<'_>,
    #[description = "Lowest number that can be picked. Defaults to 1"]
    lower: Option<i32>,
    #[description = "Highest number that can be picked. Defaults to 10"]
    upper: Option<i32>,
    #[description = "Describes what the number is for"]
    purpose: Option<String>,
    #[description = "Don't show the result to anyone else"]
    hide: Option<bool>,
) -> Result<(), Error> {
    // Defaults
    let purpose = purpose.unwrap_or("".to_string());
    let hide = hide.unwrap_or(false);
    let lower = lower.unwrap_or(1);
    let upper = upper.unwrap_or(10);

    // Buy us some more time to think; for some reason some functions can be very slow
    if hide {
        ctx.defer_ephemeral().await.unwrap();
    }else{
        ctx.defer().await.unwrap();
    }

    if upper <= lower {
        ctx.send(CreateReply::default()
            .content(format!("{} - {} is not a valid range.", lower, upper))
            .ephemeral(true)).await?;
        return Ok(())
    }

    // Generate the random number
    let result = {
        let mut generator = rng();
        let range = Uniform::try_from(lower..=upper).unwrap();
        range.sample(&mut generator)
    };

    // Format the result text
    let mut text = format!("# 🔢 {}", result);
    if !purpose.is_empty() {
        text += format!(" {}", purpose).as_str();
    }

    // Build the roll explanation text
    text += format!("\nFrom {} to {}", lower, upper).as_str();

    // Send the result to the user as an embed
    ctx.send(CreateReply::default()
        .embed(CreateEmbed::new()
            .description(text)
        ).ephemeral(hide)
    ).await?;
    Ok(())
}

/// Flip a coin
#[poise::command(slash_command, default_member_permissions = "SEND_MESSAGES")]
pub async fn coinflip(
    ctx: Context<'_>,
    #[description = "Describes what the flip is for"]
    purpose: Option<String>,
    #[description = "Don't show the result to anyone else"]
    hide: Option<bool>,
) -> Result<(), Error> {
    // Defaults
    let purpose = purpose.unwrap_or("".to_string());
    let hide = hide.unwrap_or(false);

    // Buy us some more time to think; for some reason some functions can be very slow
    if hide {
        ctx.defer_ephemeral().await.unwrap();
    }else{
        ctx.defer().await.unwrap();
    }

    // Generate the random number
    let result = {
        let mut generator = rng();
        let range = Uniform::try_from(0..=1).unwrap();
        if range.sample(&mut generator) > 0 {"Heads"} else {"Tails"}
    };

    // Format the result text
    let mut text = format!("# 🪙 {}", result);
    if !purpose.is_empty() {
        text += format!(": {}", purpose).as_str();
    }

    // Send the result to the user as an embed
    ctx.send(CreateReply::default()
        .embed(CreateEmbed::new()
            .description(text)
        ).ephemeral(hide)
    ).await?;
    Ok(())
}

/// Randomly get a Yes or No answer
#[poise::command(slash_command, default_member_permissions = "SEND_MESSAGES")]
pub async fn yesno(
    ctx: Context<'_>,
    #[description = "Describes what the decision is for"]
    purpose: Option<String>,
    #[description = "Don't show the result to anyone else"]
    hide: Option<bool>,
) -> Result<(), Error> {
    // Defaults
    let purpose = purpose.unwrap_or("".to_string());
    let hide = hide.unwrap_or(false);

    // Buy us some more time to think; for some reason some functions can be very slow
    if hide {
        ctx.defer_ephemeral().await.unwrap();
    }else{
        ctx.defer().await.unwrap();
    }

    // Generate the random number
    let result = {
        let mut generator = rng();
        let range = Uniform::try_from(1..=1000).unwrap();
        let number = range.sample(&mut generator);
        let result = if number % 2 == 0 {"Yes"} else {"No"};
        if number == 1 {"Maybe"} else {result}
    };

    // Format the result text
    let text;
    if purpose.is_empty() {
        text = format!("# ❓ {}", result);
    } else if purpose.ends_with("?") {
        text = format!("# ❓ {} {}", purpose, result);
    } else {
        text = format!("# ❓ {}? {}", purpose, result);
    }

    // Send the result to the user as an embed
    ctx.send(CreateReply::default()
        .embed(CreateEmbed::new()
            .description(text)
        ).ephemeral(hide)
    ).await?;
    Ok(())
}

/// Receive a fortune cookie
#[poise::command(slash_command, default_member_permissions = "SEND_MESSAGES")]
pub async fn fortune(
    ctx: Context<'_>,
    #[description = "Don't show the result to anyone else"]
    hide: Option<bool>,
) -> Result<(), Error> {
    // Defaults
    let hide = hide.unwrap_or(false);
    
    // Buy us some more time to think; for some reason some functions can be very slow
    if hide {
        ctx.defer_ephemeral().await.unwrap();
    }else{
        ctx.defer().await.unwrap();
    }
    

    // Check if the user is in cooldown
    let fortune_cooldown = ctx.data().fortune_cooldown;
    let previous = ctx.data().database.get_user_value(&ctx.author().id, "fortune_last").await.unwrap();
    let previous_time = ctx.data().database.get_user_value(&ctx.author().id, "fortune_last_time").await.unwrap();
    let current_time = chrono::Utc::now().timestamp();
    
    if previous_time.is_some() {
        // Convert previous_time to i64 (unix timestamp)
        let last_time = previous_time.unwrap().parse::<i64>().unwrap_or(0);

        // Check if the user is still in cooldown
        if current_time - last_time < fortune_cooldown {
            // Calculate remaining time
            let remaining = fortune_cooldown - (current_time - last_time);
            let hours = remaining / 3600;
            let minutes = (remaining % 3600) / 60;
            let seconds = remaining % 60;
            
            let mut text = format!("You must wait **{}h {}m {}s** before receiving another fortune.", hours, minutes, seconds);
            if previous.is_some() {
                text = format!("{}\nYour previous fortune was:\n> {}", text, previous.unwrap());
            }

            ctx.send(CreateReply::default()
                .embed(CreateEmbed::new()
                    .title("🥠⏳ Fortune cooldown")
                    .description(text)
                    .color(Colour::DARK_ORANGE)
                ).ephemeral(true)
            ).await?;
            return Ok(());
        }
    }

    // Generate fortune
    // fortune.json source: https://github.com/Supinic/supibot/blob/master/commands/cookie/fortune-cookies.json
    let fortune = {
        // Read the fortune.json file
        let fortune_path = std::path::Path::new("fortune.json");
        let fortune_file = std::fs::File::open(fortune_path).expect("Failed to open fortune.json");
        let fortunes: Vec<serde_json::Value> = serde_json::from_reader(fortune_file).expect("Failed to parse fortune.json");

        // Generate a random index
        let mut generator = rng();
        let range = Uniform::try_from(0..fortunes.len()).unwrap();
        let index = range.sample(&mut generator);

        // Extract the fortune text
        fortunes[index]["text"].as_str().unwrap_or("Your future is cloudy at the moment.").to_string()
    };

    // Store current time and fortune in the database
    ctx.data().database.set_user_value(&ctx.author().id, "fortune_last", &fortune).await?;
    ctx.data().database.set_user_value(&ctx.author().id, "fortune_last_time", &current_time.to_string()).await?;

    // Send the fortune to the user
    ctx.send(CreateReply::default()
        .embed(CreateEmbed::new()
            .title("🥠 Your Fortune")
            .description(fortune)
            .color(Colour::GOLD)
        ).ephemeral(hide)
    ).await?;
    Ok(())
}
