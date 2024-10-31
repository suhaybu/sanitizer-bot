use crate::{Data, Error};

pub mod age;

pub fn get_all() -> Vec<poise::Command<Data, Error>> {
    vec![age::age()]
}
