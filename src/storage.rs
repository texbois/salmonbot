use crate::BotResult;
use std::sync::Mutex;

pub struct Storage {
    redis: Mutex<redis::Connection>,
}

impl Storage {
    pub fn new(redis_url: &str) -> BotResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            redis: Mutex::new(client.get_connection()?),
        })
    }
}
