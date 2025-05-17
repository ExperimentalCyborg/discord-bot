# Discord Bot
It's a Discord bot 🙂

Note: Crossed out items are not yet implemented, but planned.

## Features

### Utilities
Random generator:
- D&D-style dice roll (e.g. 2d12+4). It rolls one 6-sided die if no options are given.
- Random number picker with custom range
- Coin flip
- Yes/No

Resources
- Fortune - get a random fortune cookie.
- ~~Image search - get a random image from Unsplash based on given search terms.~~

### Administrative
- Message edit/delete tracking
- User join/leave tracking
- ~~Userprofile edit tracking~~
- ~~Audit log tracking~~
- ~~Reaction roles~~

Event logs are sent to whichever text channel you specify. Each tracking feature has its own settings and can share the 
same channel with other features if desired. Only a server admin can configure the bot.

## Privacy
The bot only communicates with the Discord gateway servers. You, the operator, have full control over the data.

## Data storage
The bot uses an embedded SQLite database which stores everything in a local file. 

## Running locally
Install the Rust programming language compiler:
https://rustup.rs/

Build the executable:
```shell
cargo build --release
```

Run the bot:
```shell
./target/release/discord-bot --db-path <path_here> --bot-token <token_here>
```
`--db-path` must refer to a directory. It will create the database file on its own. Run `discord-bot` with `--help` to 
see all the available options.

If you don't have a bot token, follow [these instructions](#set-up-your-discord-application-bot-account)

## Running in Docker

### First time setup:
#### Set up your Discord application (bot account)
If you haven't already, create a new Discord application [here](https://discord.com/developers/applications/) 
in the Discord Developer Portal. Reset your (newly created) application's bot token in the `settings/bot` menu and 
paste it into the `.env` file you'll create next.

#### Define your settings
Copy `.env.template` and name it `.env`. Follow the instructions in this file and adjust the settings to your liking. 
Don't forget to fill in the Discord bot token.

#### Run the bot
```shell
# Download and/or build missing resources if needed, and then run it 
sudo docker compose up -d
```
The first start may take a while, since the bot will be built from source code. 
This ensures that it will run on any architecture as long as Docker and Rust support it.
This bot runs great on a Raspberry Pi!

### Stopping the bot without data loss
```shell
# Send the container the signal to shut down gracefully
sudo docker compose down
```

### Updating the bot to the latest version without data loss and minimal downtime
```shell
# Update the code (skip this if you want to apply local code changes)
git pull
# Build an updated docker image
sudo docker compose build --no-cache
# Replace the currently running container with the newly built one
sudo docker compose up -d
```
