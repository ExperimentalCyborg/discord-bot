use rand::distributions::Distribution;
use log::{debug, info, warn, error};
use poise::CreateReply;
use poise::serenity_prelude::{Colour, CreateEmbed, CreateEmbedFooter};
use crate::{Context, Data, Error};
use chrono::Local;
use clap::builder::TypedValueParser;
use regex::Regex;
use rand::{thread_rng, Rng};
use rand::distributions::Uniform;

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

/// Check if the bot is still alive
///
/// Shows timing-related real time statistics about the bot.
#[poise::command(slash_command)]
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
            .field("Bot date/time", format!("{}", now.to_rfc2822()), false)
            .color(colour)
        )
    ).await?;
    Ok(())
}

/// Get detailed info about the bot
#[poise::command(slash_command)]
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
            .field("Bot boot time", ctx.data().time_started.to_rfc2822(), false)
            .field("Bot local date/time", now.to_rfc2822(), false)
            .footer(CreateEmbedFooter::new("Note: avatar is set by the bot owner, not the software author."))
        )
    ).await?;
    Ok(())
}

/// Show this help menu
#[poise::command(slash_command)]
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
/// Rolls between 1 and 6 when no `dice` are specified.
/// Rolls are calculated individually, meaning the odds are identical to rolling the given amount of physical dice.
#[poise::command(slash_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "How many dice to roll"]
    rolls: Option<u8>,
    #[description = "How many sides each die has"]
    sides: Option<u8>,
    #[description = "How much to add to or subtract from the total"]
    offset: Option<i32>,
    #[description = "What the roll is for"]
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
        let mut rng = thread_rng();
        let range = Uniform::from(1..sides);
        for _ in 0..rolls {
            results.push(range.sample(&mut rng) as i32);
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
            .color(Colour::LIGHTER_GREY)
        ).ephemeral(hide)
    ).await?;
    Ok(())
}
