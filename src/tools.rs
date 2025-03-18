use std::num::NonZeroU64;

const YEAR_LENGTH:f64 = 365.242189;

pub fn to_snowflake(snowflake_string: &str) -> Option<NonZeroU64> {
    snowflake_string.parse::<NonZeroU64>().ok()
}

pub fn to_channel(snowflake_string: &str)
    -> Option<poise::serenity_prelude::model::id::ChannelId> {
    let snowflake = match to_snowflake(snowflake_string){
        Some(s) => s,
        None => return None,
    };
    Some(poise::serenity_prelude::model::id::ChannelId::from(snowflake))
}

pub fn user_account_age(user_id: poise::serenity_prelude::UserId) -> String {
    // Get the timestamp from the user ID
    let timestamp = user_id.created_at().timestamp();
    let creation_date = match chrono::DateTime::from_timestamp(timestamp, 0){
        Some(d) => d,
        None => return String::from("Unknown"),
    };

    let duration = chrono::Utc::now().signed_duration_since(creation_date);
    let years = duration.num_days() as f64 / YEAR_LENGTH;
    let days = duration.num_days() as f64 % YEAR_LENGTH;
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;
    format!("{:.0} years, {:.0} days, {} hours, {} minutes, {} seconds",
            years, days, hours, minutes, seconds)
}
