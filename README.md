# Discord Bot
It's a Discord bot 🙂

Note: Crossed out items are not yet implemented, but planned.

## Features

### Utilities
Random generator:
- D&D-style dice roll (e.g. 2d12+4). It rolls one 6-sided die if no options are given.
- ~~Random number picker with custom range~~
- ~~Coin flip~~
- ~~Yes/No~~

Resources
- ~~Image search - get a random image from Unsplash based on given search terms.~~
- ~~Fortune - get a random fortune cookie.~~
- ~~Quote - get a random quote.~~

### Administrative
- Message edit/delete tracking
- User join/leave tracking
- ~~Userprofile edit tracking~~
- ~~Audit log tracking~~

Event logs are sent to whichever text channel you specify. Each tracking feature has its own settings and can share the 
same channel with other features if desired.

### ~~AI~~
If you have an OpenAI API compatible endpoint available (for example your own ollama, an OpenAI subscription, or a cloud service), you can enable AI features.
- When a user @ the bot with a question in a text channel, it'll join the conversation (until it is told to leave or gets ignored for a while).
- While participating in conversations, the bot will learn and remember the personality of participating users. It will then apply this in later conversations.
- Ask the bot one-off questions with a / command. It won't read chat or store any messages.
- User access to AI features can be controlled by configuring an opt-in role. If configured, users without this role will be ignored by the bot and no information will be stored about them.
- Optionally, AI features can require the server admin to enter their own API endpoint / credentials. This is globally disabled by default for security, since this can leak the IP of the machine this bot runs on.
- Optionally, AI features can be allowlisted for specific discord servers only.

## Privacy
The bot only communicates with the Discord gateway servers ~~, and optionally the OpenAI-compatible REST api you've 
configured~~. You, the operator, have full control over the data ~~, and through the opt-in role for AI, so do your 
discord guild members.~~

## Data storage
By default, the bot uses an embedded SQLite database which stores everything in a local file. 
~~Optionally an external database like PostgreSQL can be used.~~ (todo)

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
