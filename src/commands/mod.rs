use crate::{Data, Error};

mod age;
mod config;
mod credits;
mod sanitize;

pub fn get_all() -> Vec<poise::Command<Data, Error>> {
    vec![
        age::age(),
        sanitize::sanitize(),
        sanitize::sanitize_menu(),
        config::config(),
        credits::credits(),
    ]
}
