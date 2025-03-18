use std::num::NonZeroU16;
use log::{debug, info, warn, error};
use poise::serenity_prelude::{Colour, CreateEmbed, Mentionable, Message, MessageRef};
use poise::serenity_prelude::model::Timestamp;
use poise::serenity_prelude::CreateMessage;
use poise::serenity_prelude::EntityType::Str;
use crate::serenity;
use crate::{Context, Data, Error};

pub async fn event_dispatcher(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!("Authenticated as {} ID {}", data_about_bot.user.name, data_about_bot.user.id);
        }
        serenity::FullEvent::GuildCreate { guild, is_new, } => {
            poise::builtins::register_in_guild(ctx, &framework.options().commands, guild.id).await?;
            let seen_before = data.database.get_guild_value(&guild.id, &"stats.first_join").await?;
            if seen_before.is_none(){
                data.database.set_guild_value(&guild.id, &"stats.first_join", &Timestamp::now()).await?;
                info!("Joined guild \"{}\" ID {}", guild.name, guild.id);
            }else{
                info!("Reconnected to guild \"{}\" ID {}", guild.name, guild.id);
            }
            data.database.set_guild_value(&guild.id, &"stats.last_join", &Timestamp::now()).await?;
            data.database.set_guild_value(&guild.id, &"stats.name", &guild.name).await?;
        }
        serenity::FullEvent::GuildDelete { incomplete, full } => {
            let name = match full { // Get last known guild name from db if we haven't cached it
                Some(n) => n.name.clone(),
                None => data.database.get_guild_value(&incomplete.id, &"stats.name").await?.unwrap_or("Unknown".to_string()),
            };

            match incomplete.unavailable {
                true => info!("Guild unavailable: \"{}\" ID {}", name, incomplete.id),
                false => {
                    data.database.set_guild_value(&incomplete.id, &"stats.kicked_from", &"").await?;
                    info!("Removed from guild: \"{}\" ID {}", name, incomplete.id)
                }
            }
        }
        serenity::FullEvent::MessageDeleteBulk { channel_id, multiple_deleted_messages_ids, guild_id } => {
            // todo
        }
        serenity::FullEvent::MessageDelete { channel_id, deleted_message_id, guild_id } => {
            // Exit if it's not a guild message
            if guild_id.is_none(){
                return Ok(());
            }
            let guild_id = guild_id.unwrap();

            // Exit if we don't log these
            let _lci = data.database.get_guild_value(&guild_id, &"config.track_msg_edits").await;
            if _lci.is_err(){
                return Ok(());
            }

            // todo make a helper function to convert snowflake strings to channel IDs, because we'll be doing this a dozen more times
            let log_channel_id = data.database.to_snowflake(_lci.ok().unwrap().unwrap().as_str()).unwrap();
            let log_channel_id = poise::serenity_prelude::model::id::ChannelId::from(log_channel_id);

            let (author, content, is_available) = {
                let message = ctx.cache.message(channel_id, deleted_message_id);
                match message {
                    None => ("Unknown".to_string(), "".to_string(), false),
                    Some(m) => (
                        m.author.mention().to_string(),
                        m.content.clone(),
                        true
                    ),
                }
            };

            // Log the message
            log_channel_id.send_message(&ctx.http,
                 CreateMessage::new().embed(
                    CreateEmbed::new()
                        .title("💬🗑️ Message deleted")
                        .field("ID:", deleted_message_id.to_string(), false)
                        .field("Channel:", channel_id.mention().to_string(), false)
                        .field("Author:", author, false)
                        .field({if is_available{
                                    "Message:"
                                } else {
                                    "Message content unavailable."
                                }
                               }, content, false)
                        .color(Colour::DARK_RED)
                    )
                ).await?;
        }
        serenity::FullEvent::MessageUpdate { old_if_available, new, event } => {
            if new.is_none(){ // todo this happens when an uncached message is edited...
                warn!("Failed to log edit due to a cache miss: {}", event.id);
                return Ok(());
            }

            let new_message = new.clone().unwrap();
            let old_message = old_if_available.clone();
            let guild_id = new_message.guild_id;

            // Exit if it's not a guild message
            if guild_id.is_none(){
                return Ok(());
            }
            let guild_id = guild_id.unwrap();

            // Exit if we don't log these
            let _lci = data.database.get_guild_value(&guild_id, &"config.track_msg_edits").await;
            if _lci.is_err(){
                return Ok(());
            }

            // todo make a helper function to convert snowflake strings to channel IDs, because we'll be doing this a dozen more times
            let log_channel_id = data.database.to_snowflake(_lci.ok().unwrap().unwrap().as_str()).unwrap();
            let log_channel_id = poise::serenity_prelude::model::id::ChannelId::from(log_channel_id);

            let (is_available, old_content) = {
                match old_message {
                    None => (false, String::new()),
                    Some(m) => (true, m.content.clone())
                }
            };

            // Log the message
            log_channel_id.send_message(&ctx.http,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .title("💬✏️ Message edited")
                        .field("Author:", new_message.author.to_string(), false)
                        .field("Channel:", new_message.channel_id.mention().to_string(), false)
                        .field("Message:", new_message.link().to_string(), false)
                        .field({
                                   if is_available{
                                       "Old:"
                                   }else {
                                       "Old message content unavailable."
                                   }
                               }, old_content, false)
                        .field("New:", new.clone().unwrap().content, false)
                        .color(Colour::DARK_TEAL)
                )
            ).await?;
            // todo use a diff instead of the full message twice
        }
        serenity::FullEvent::Message { new_message } => {
            // TODO
        }
        serenity::FullEvent::GuildBanAddition { guild_id, banned_user } => {
            // TODO
        }
        serenity::FullEvent::GuildBanRemoval { guild_id, unbanned_user } => {
            // todo
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            // todo
        }
        serenity::FullEvent::GuildMemberRemoval { guild_id, user, member_data_if_available } => {
            // todo
        }
        serenity::FullEvent::GuildMemberUpdate { old_if_available, new, event } => {
            // todo
        }
        serenity::FullEvent::GuildUpdate { old_data_if_available, new_data } => {
            data.database.set_guild_value(&new_data.id, &"stats.name", &new_data.name).await?;
        }
        serenity::FullEvent::GuildAuditLogEntryCreate { guild_id, entry } => {

        }
        serenity::FullEvent::Resume { event } => {
            info!("Reconnected to gateway");
        }
        _ => {
            debug!("Received unhandled event: {:?}", event.snake_case_name());
        }
    }
    Ok(())
}
