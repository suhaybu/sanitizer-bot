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

To host your own instance on Shuttle, follow along. This project is not affiliated or sponsored by Shuttle. This setup assumes you have Rust installed on your machine.

0. Optional: Create an account on Shuttle [here](https://www.shuttle.dev/), and then follow the documentation for setting up the Shuttle cli [here](https://docs.shuttle.dev/getting-started/installation).

1. Clone this repository.

2. Rename `Secrets.dev.toml.example` to `Secrets.toml` in the project root and update the following variables with the following:

| Variable | Description | Where to get it |
|----------|-------------|-----------------|
| `DISCORD_TOKEN` | Discord bot token for authentication | [Discord Developer Portal](https://discord.com/developers/applications) |
| `TURSO_DATABASE_URL` | Database URL for storing server configurations | [Turso Dashboard](https://turso.tech/) |
| `TURSO_AUTH_TOKEN` | Authentication token for Turso database access | [Turso Dashboard](https://turso.tech/) |

3. Deploy the code using ```cargo shuttle deploy```

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
