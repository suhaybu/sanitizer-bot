use crate::{Data, Error};

mod age;
mod sanitize;

use age::age;
use sanitize::sanitize;

pub fn get_all() -> Vec<poise::Command<Data, Error>> {
    vec![age(), sanitize()]
}
