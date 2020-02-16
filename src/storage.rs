use crate::BotResult;
use redis::Commands;
use std::sync::Mutex;

pub type StorageResult<'s, T> = Result<T, Box<dyn std::error::Error + 's>>;

pub struct Storage {
    redis: Mutex<redis::Connection>,
}

impl Storage {
    pub fn new(redis_url: &str) -> BotResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let redis = client.get_connection()?;

        Ok(Self {
            redis: Mutex::new(redis),
        })
    }

    pub fn set_add<'s, V: redis::ToRedisArgs + std::fmt::Display + Copy>(
        &'s self,
        set: &str,
        value: V,
    ) -> StorageResult<'s, u64> {
        let mut conn = self.redis.lock()?;
        match conn.sadd(set, value) {
            Ok(()) => conn.scard(set).map_err(|e| e.into()),
            Err(e) => Err(format!("Cannot add {} to {}: {}", value, set, e).into()),
        }
    }

    pub fn set_contains<'s, V: redis::ToRedisArgs + std::fmt::Display + Copy>(
        &'s self,
        set: &str,
        value: V,
    ) -> StorageResult<'s, bool> {
        let mut conn = self.redis.lock()?;
        conn.sismember(set, value)
            .map_err(|e| format!("Cannot check membership of {} in {}: {}", value, set, e).into())
    }
}
