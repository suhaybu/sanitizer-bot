# Sanitizer Bot

## Introduction

Sanitizer is a simple bot that uses regex to identify links for social platforms and replaces them with discord embed friendly links that allows you to view the content of the link without ever having to leave the discord app. Making peoples lives easier one step at a time! :)

The bot is developed using the `twilight` library. You can view their GitHub repo [here](https://github.com/twilight-rs/twilight). You can add the Sanitizer bot to your server or install it as a user app from [here](https://discord.com/oauth2/authorize?client_id=1197778683113513081).

This project was first written in Python and that version can be found [here](https://github.com/Suhaybu/sanitizer-bot-py).

## Features

-   **No logs:** No form of logs are saved.
-   **Supports Multiple platforms:** Currently works with Twitter, TikTok, and Instagram.
-   **Configurable:** You can change the behavior of the bot using `/config`.
-   **User Installable App:** The `/sanitize` app command can be used anywhere.
-   **Handles Direct Messages:** Will attempt to fix links sent directly in DM's.


## Setup

To host your own instance of the bot on Shuttle, follow along. This project is not affiliated or sponsored by Shuttle, and assumes you have Rust installed on your machine.

1. Download the release based on your OS and CPU architecture.

2. Set the following variables to your enviornment, or in a `.env` file in the same directory as the binary:

| Variable | Description | Where to get it |
|----------|-------------|-----------------|
| `DISCORD_TOKEN` | Discord bot token for authentication | [Discord Developer Portal](https://discord.com/developers/applications) |
| `TURSO_DATABASE_URL` | Database URL for storing server configurations | [Turso Dashboard](https://turso.tech/) |
| `TURSO_AUTH_TOKEN` | Authentication token for Turso database access | [Turso Dashboard](https://turso.tech/) |
| `EMOJI_ID` | Emoji ID used by the bot to react to messages. | [Discord](https://discord.com/developers/applications/) |

3. Run the binary.

### Docker

Sanitizer can also be run using Docker. Prebuilt, lightweight images are published to GitHub Container Registry (GHCR) on each release.

Image:
`ghcr.io/suhaybu/sanitizer-bot`

The container uses the same environment variables as the binary:
DISCORD_TOKEN, TURSO_DATABASE_URL, TURSO_AUTH_TOKEN, EMOJI_ID

When running in Docker, Sanitizer uses Turso’s embedded replica mode. This creates a local SQLite database file inside the container that must be persisted with a volume mount so server configuration and message mappings survive restarts.

A sample docker-compose.yml is provided in the repository for reference. Note that the example docker-compose.yml references the `stack.env` file used within Portainer. You can modify this to use your own `.env` file or set the variables directly in the docker-compose.yml if you wish.

To update the bot:
- Pull the latest image from GHCR
- Restart the container


## Usage

Once the bot is running, you can use the following commands:
-   `/credits`: To roll the credits
-   `/sanitize`: To fix the embed of your link
-   `/config`: to configure the bot

## License

Distributed under the MIT License. See `LICENSE` for more information.

## Contact

Twitter - [@suhayb_u](https://twitter.com/suhayb_u)

## Acknowledgments
-   [twilight](https://github.com/twilight-rs/twilight)
-   [FxTwitter](https://github.com/FixTweet/FxTwitter)
-   [kkScript](https://kkscript.com/)
-   [InstaFix](https://github.com/Wikidepia/InstaFix)
-   [FxTwitch](https://github.com/seriaati/fxtwitch)
-   [FxReddit](https://github.com/MinnDevelopment/fxreddit)
-   ~~[QuickVids](https://quickvids.app/)~~ (No longer used)
