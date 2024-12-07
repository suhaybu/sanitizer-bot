use crate::{Data, Error};

mod config;
mod credits;
mod sanitize;

pub fn get_all() -> Vec<poise::Command<Data, Error>> {
    vec![
        sanitize::sanitize_slash(),
        sanitize::sanitize_menu(),
        // config::config(),
        credits::credits(),
    ]
}
