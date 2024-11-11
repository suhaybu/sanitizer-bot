use crate::{Data, Error};

mod age;
mod config;
mod credits;
mod sanitize;

use age::age;
use config::config;
use sanitize::sanitize;

pub fn get_all() -> Vec<poise::Command<Data, Error>> {
    vec![age(), sanitize(), config()]
}
