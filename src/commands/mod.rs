use crate::{Data, Error};

mod credits;
mod sanitize;
mod settings;

pub fn get_all() -> Vec<poise::Command<Data, Error>> {
    vec![
        sanitize::sanitize_slash(),
        sanitize::sanitize_menu(),
        settings::settings(),
        credits::credits(),
    ]
}
