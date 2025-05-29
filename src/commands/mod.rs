use crate::{Data, Error};

mod credits;
mod sanitize;
mod config;

pub fn get_all() -> Vec<poise::Command<Data, Error>> {
    vec![
        sanitize::sanitize_slash(),
        sanitize::sanitize_menu(),
        credits::credits(),
        config::config(),
    ]
}
