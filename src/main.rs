use log::{debug, info, warn, error, LevelFilter};
use clap::{Parser, crate_version, crate_description, crate_authors};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::model::Timestamp;
use tokio::signal;
use crate::database::Database;

mod commands;
mod events;
mod database;
mod ai;
mod tools;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Parser, Debug)]
#[command(version, about, author)]
struct Args {
    /// Directory path to SQLite db storage
    #[arg(short, long)]
    db_path: String,

    /// Discord bot token
    #[arg(short, long)]
    bot_token: String,

    /// Log level for the CLI. Default is Info.
    #[arg(long, value_enum)]
    log_level: Option<LevelFilter>,

    /// Log file path. When omitted, no log file is used.
    #[arg(long)]
    log_file_path: Option<String>,

    /// Log level for the log file
    #[arg(long, value_enum)]
    log_file_level: Option<LevelFilter>,

    /// OpenAI compatible API URL. AI functions disabled when omitted.
    #[arg(long)]
    openai_api_root: Option<String>,

    /// API token for OpenAI compatible API, if required.
    #[arg(long)]
    openai_api_token: Option<String>,
}

// Custom user data passed to all command functions
pub struct Data {
    // Read only attributes
    time_started: Timestamp,
    //args: Args, // todo remove?
    app_version: String,
    app_description: String,
    app_authors: String,
    database: Database,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    env_logger::Builder::default()
        .filter_level(LevelFilter::Error) // Everything except our own code only prints >= error
        .filter_module("discord_bot", args.log_level.unwrap_or(LevelFilter::Info)) // Our own code
        .init();
    let time_started = Timestamp::now();
    info!("Application starting");
    debug!("Arguments: {:?}", args);

    let db_path = args.db_path.clone();
    let db;
    match Database::new(&db_path).await {
        Ok(d) => {
            db = d;
        }
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    }

    debug!("Configuring Poise");
    // FrameworkOptions contains all of poise's configuration options in one struct
    let options = poise::FrameworkOptions {
        commands: vec![
            commands::info(),
            commands::help(),
            commands::ping(),
            commands::roll(),
            commands::number(),
            commands::coinflip(),
            commands::yesno(),
            commands::track(),
        ],

        prefix_options: poise::PrefixFrameworkOptions {
            ..Default::default()
        },

        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),

        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                commands::pre_command(ctx)
            })
        },

        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                commands::post_command(ctx)
            })
        },

        // Every command invocation must pass this check to continue execution
        command_check: Some(|ctx| {
            Box::pin(async move {
                commands::command_check(ctx)
            })
        }),

        // Enforce command checks even for owners
        skip_checks_for_owners: false,

        event_handler: |ctx, event, framework, data| {
            Box::pin(events::event_dispatcher(ctx, event, framework, data))
        },

        ..Default::default()
    };

    debug!("Gathering client settings");
    let token = args.bot_token.clone();
    let intents = serenity::GatewayIntents::non_privileged() |
        serenity::GatewayIntents::MESSAGE_CONTENT | serenity::GatewayIntents::GUILD_MEMBERS;

    debug!("Initializing globally shared data");
    let global_data = Data {
        time_started,
        // args, // todo remove?
        app_version: crate_version!().to_string(),
        app_description: crate_description!().to_string(),
        app_authors: crate_authors!("\n").to_string(),
        database: db,
    };

    debug!("Setting up Serenity client");
    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| { Box::pin(async move { Ok(global_data) }) })
        .options(options)
        .build();
    let mut client;
    match serenity::ClientBuilder::new(token, intents).framework(framework).await {
        Ok(c) => {
            client = c;
        }
        Err(e) => {
            error!("Failed to initialize client: {}", e);
            std::process::exit(1);
        }
    }
    client.cache.set_max_messages(5000);

    // Set up SIGTERM handling (Unix-only)
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to register SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>(); // Never resolves on non-Unix platforms

    info!("Connecting to Discord");
    // Create a future that completes when we receive either Ctrl+C/SIGINT or SIGTERM
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            warn!("Received SIGINT, shutting down...");
            client.shard_manager.shutdown_all().await;
        }
        _ = terminate => {
            warn!("Received SIGTERM, shutting down...");
            client.shard_manager.shutdown_all().await;
        }

        _ = client.start_autosharded() => {
            warn!("Exited.");
        }
    }
}
