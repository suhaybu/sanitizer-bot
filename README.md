# Sanitizer Bot (Rust)

This project is a Rust implementation of [Sanitizer Bot (Python)](https://github.com/Suhaybu/sanitizer-bot-py)

## Introduction

Sanitizer is a simple bot that uses regex to identify links for social platforms and replaces them with discord embed friendly links that allows you to view the content of the link without ever having to leave the discord app. Making peoples lives easier one step at a time! :)

The bot is developed using the `serenity.rs` library. Click [here](https://github.com/serenity-rs/serenity) to view their GitHub repo.

## Features

-   **No logs:** No form of logs are saved.
-   **Supports Multiple platforms:** Currently works with Twitter, TikTok, and Instagram.
-   **Configurable:** You can change the behavior of the bot using `/config`.
-   **User Installable App:** The `/sanitize` app command can be used anywhere.
-   **Handles Direct Messages:** Will attempt to fix links sent directly in DM's.
-   **Uses QuickVids API:** Uses QuickVids API to convert TikTok links into embeddable content in discord.


## Build
```cargo run --release```

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
-   [serenity.rs](https://github.com/serenity-rs/serenity)
-   [FxTwitter](https://github.com/FixTweet/FxTwitter)
-   [QuickVids](https://quickvids.app/)
-   [InstaFix](https://github.com/Wikidepia/InstaFix)
