# Sanitizer Bot (Rust)

### ⚠️ This project is still in early development ⚠️
This project is a Rust implementation of [Sanitizer Bot (Python)](https://github.com/Suhaybu/sanitizer-bot-py)

## Introduction

Sanitizer is a simple bot that uses regex to identify links for social platforms and replaces them with discord embed friendly links that allows you to view the content of the link without ever having to leave the discord app. You might think this is a bot for lazy people, but I assure you, if you give it a try, you'll never want to go back.

The bot is developed in Rust using the `serenity.rs` library. Click [here](https://github.com/serenity-rs/serenity) for their GitHub repo. This GitHub repo is utilized for version control.

## TODO

-   EVERYTHING
-   Add Reddit Support
-   Add a config panel

## Features

-   **User Privacy first:** No logs are made on any messages users send.
-   **Supports Multiple platforms:** Currently works with Twitter, TikTok, Instagram. More to come!
-   **Automatic conversion:** Automatically fixes any supported links posted.
-   **User Installable App:** The `/sanitize` app command can be used anywhere.
-   **Handles Direct Messages:** Will attempt to fix links sent in private.
-   **Implemented QuickVids API:** Implemented QuickVids API to convert TikTok links into embeddable content in discord.


## Build
```colima stop && colima start --cpu 4 --memory 4 && COMPOSE_HTTP_TIMEOUT=300 docker-compose up --build```

## Usage

Once the bot is running, you can use the following commands:
-   `/credits`: To roll the credits
-   `/sanitize`: To fix the embed of your link
## Contributing

Contributions are what make the open-source community such an amazing place to learn, inspire, and create.


## License

Distributed under the MIT License. See `LICENSE` for more information.

## Contact

Twitter - [@suhayb_u](https://twitter.com/suhayb_u)

## Acknowledgments
-   [serenity.rs](https://github.com/serenity-rs/serenity)
