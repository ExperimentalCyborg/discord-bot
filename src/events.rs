use std::num::NonZeroU16;
use log::{debug, info, warn, error};
use poise::serenity_prelude::{Colour, CreateEmbed, Mentionable, Message, MessageRef};
use poise::serenity_prelude::model::Timestamp;
use poise::serenity_prelude::CreateMessage;
use poise::serenity_prelude::EntityType::Str;
use crate::{serenity, tools};
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

            // Exit if we don't log these // todo de-duplicate, we're doing this many times.
            let _lci = match data.database.get_guild_value(&guild_id, &"config.track_msg_edits").await {
                Ok(lci) => match lci {
                    Some(l) => l,
                    None => return Ok(())},
                Err(_) => return Ok(())
            };

            let (author, content, is_available) = {
                let message = ctx.cache.message(channel_id, deleted_message_id);
                match message {
                    None => ("Unknown".to_string(), "".to_string(), false),
                    Some(m) => {
                        if m.author.bot{
                            return Ok(()); // Ignore bots
                        }
                        (
                            m.author.mention().to_string(),
                            m.content.clone(),
                            true
                        )
                    },
                }
            };

            // Log the message
            let log_channel_id = tools::to_channel(_lci.as_str()).unwrap();
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
            if new.is_none(){
                warn!("Failed to log edit due to a cache miss: {}", event.id);
                return Ok(());
            }

            if new.clone().unwrap().author.bot{
                return Ok(()); // Ignore messages from bots
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
            let _lci = match data.database.get_guild_value(&guild_id, &"config.track_msg_edits").await {
                Ok(lci) => match lci {
                    Some(l) => l,
                    None => return Ok(())},
                Err(_) => return Ok(())
            };

            let (is_available, old_content) = {
                match old_message {
                    None => (false, String::new()),
                    Some(m) => (true, m.content.clone())
                }
            };
            let new_content = new_message.content.clone();

            if is_available && old_content == new_content{
                return Ok(());
            }

            // Log the message
            let log_channel_id = tools::to_channel(_lci.as_str()).unwrap();
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
                        .field("New:", new_content, false)
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
            let guild_id = new_member.guild_id;

            // Exit if we don't log these
            let _lci = match data.database.get_guild_value(&guild_id, &"config.track_joinleaves").await {
                Ok(lci) => match lci {
                    Some(l) => l,
                    None => return Ok(())},
                Err(_) => return Ok(())
            };

            let discriminator = match new_member.user.discriminator {
                Some(d) => d.to_string(),
                None => String::from("None (next gen account)")
            };

            // Log the message
            let log_channel_id = tools::to_channel(_lci.as_str()).unwrap();
            log_channel_id.send_message(&ctx.http,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .title("👋 User joined")
                        .field("User:", new_member.user.mention().to_string(), false)
                        .field("ID:", new_member.user.id.to_string(), false)
                        .field("User name:", new_member.user.name.to_string(), false)
                        .field("Discriminator:", discriminator, false)
                        .field("Global nickname:", new_member.clone().user.global_name.unwrap_or(String::from("")), false)
                        .field("Account age:", tools::user_account_age(new_member.user.id), false)
                        .thumbnail(new_member.user.avatar_url().unwrap_or(String::from("")))
                        .color(Colour::DARK_GREEN)
                )
            ).await?;
        }
        serenity::FullEvent::GuildMemberRemoval { guild_id, user, member_data_if_available } => {
            // Exit if we don't log these
            let _lci = match data.database.get_guild_value(&guild_id, &"config.track_joinleaves").await {
                Ok(lci) => match lci {
                    Some(l) => l,
                    None => return Ok(())},
                Err(_) => return Ok(())
            };

            let (server_nickname, member_age) = match member_data_if_available {
                Some(m) => (
                    m.clone().nick.unwrap_or(String::from("")),
                    {
                        match m.clone().joined_at{
                            Some(s) => {
                                let duration = chrono::Utc::now().signed_duration_since(chrono::DateTime::from_timestamp(s.unix_timestamp(), 0).unwrap());
                                let days = duration.num_days();
                                let hours = duration.num_hours() % 24;
                                let minutes = duration.num_minutes() % 60;
                                let seconds = duration.num_seconds() % 60;
                                format!("{} days, {} hours, {} minutes, {} seconds", days, hours, minutes, seconds)
                            },
                            None => String::from("Unknown") // join timestamp unavailable
                        }
                    }
                ),
                None => (String::from(""), String::from("Unknown")) // member data unavailable
            };

            let discriminator = match user.discriminator {
                Some(d) => d.to_string(),
                None => String::from("None (next gen account)")
            };

            // Log the message
            let log_channel_id = tools::to_channel(_lci.as_str()).unwrap();
            log_channel_id.send_message(&ctx.http,
                CreateMessage::new().embed(
                    CreateEmbed::new()
                        .title("🚪 User left")
                        .field("User:", user.mention().to_string(), false)
                        .field("ID:", user.id.to_string(), false)
                        .field("User name:", user.name.to_string(), false)
                        .field("Discriminator:", discriminator, false)
                        .field("Global nickname:", user.clone().global_name.unwrap_or(String::from("")), false)
                        .field("Server nickname:", server_nickname, false)
                        .field("Server membership age:", member_age, false)
                        .field("Account age:", tools::user_account_age(user.id), false)
                        .thumbnail(user.avatar_url().unwrap_or(String::from("")))
                        .color(Colour::DARK_RED)
                )
            ).await?;
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
