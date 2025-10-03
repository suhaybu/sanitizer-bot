# Sanitizer Bot

## Introduction

Sanitizer is a simple bot that uses regex to identify links for social platforms and replaces them with discord embed friendly links that allows you to view the content of the link without ever having to leave the discord app. Making peoples lives easier one step at a time! :)

The bot is developed using the `twilight` library. You can view their GitHub repo [here](https://github.com/twilight-rs/twilight).

This project was first written in Python and can be found [here](https://github.com/Suhaybu/sanitizer-bot-py).

## Features

-   **No logs:** No form of logs are saved.
-   **Supports Multiple platforms:** Currently works with Twitter, TikTok, and Instagram.
-   **Configurable:** You can change the behavior of the bot using `/config`.
-   **User Installable App:** The `/sanitize` app command can be used anywhere.
-   **Handles Direct Messages:** Will attempt to fix links sent directly in DM's.


## Setup

1. Create a `.env` file in the project root or add the following environment variables:

| Variable | Description | Where to get it |
|----------|-------------|-----------------|
| `DISCORD_TOKEN` | Discord bot token for authentication | [Discord Developer Portal](https://discord.com/developers/applications) |
| `TURSO_DATABASE_URL` | Database URL for storing server configurations | [Turso Dashboard](https://turso.tech/) |
| `TURSO_AUTH_TOKEN` | Authentication token for Turso database access | [Turso Dashboard](https://turso.tech/) |

2. Download the release compatible with your OS and run it!

3. Optionally, if you wish to build it yourself instead:
```fish
git clone https://github.com/suhaybu/sanitizer-bot
cd /sanitizer-bot
cargo run --release
```

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
-   ~~[QuickVids](https://quickvids.app/)~~ (No longer used)
