use log::{debug, info, warn, error};
use poise::serenity_prelude::Message;
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
            info!("Joining guild \"{}\" ID {}", guild.name, guild.id);
            poise::builtins::register_in_guild(ctx, &framework.options().commands, guild.id).await?;
        }
        serenity::FullEvent::GuildDelete { incomplete, full } => {
            let name;
            
            match full {
                Some(n) => name = n.name.clone(), 
                None => {name = "<UNKNOWN>".to_string();}
            }

            match incomplete.unavailable {
                true => info!("Guild unavailable: \"{}\" ID {}", name, incomplete.id),
                false => info!("Removed from guild: \"{}\" ID {}", name, incomplete.id)
            }
        }
        serenity::FullEvent::MessageDeleteBulk { channel_id, multiple_deleted_messages_ids, guild_id } => {
            // todo
        }
        serenity::FullEvent::MessageDelete { channel_id, deleted_message_id, guild_id } => {
            // todo
        }
        serenity::FullEvent::MessageUpdate { old_if_available, new, event } => {
            // todo
        }
        serenity::FullEvent::Message { new_message } => {
            message_received(ctx, new_message).await;
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
        _ => {
            debug!("Received unhandled event: {:?}", event.snake_case_name());
            // todo log as debug
        }
    }
    Ok(())
}

async fn message_received(ctx: &serenity::Context, message: &Message) {
    if message.content.to_lowercase().contains("cheese")
        && message.author.id != ctx.cache.current_user().id
    {
        message.reply(ctx, "😋🧀").await.err();
    }
}

// TODO async functions here for handling events with complex operations