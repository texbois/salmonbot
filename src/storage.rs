use crate::BotResult;
use redis::Commands;
use std::{ops::DerefMut, sync::Mutex};

pub type StorageResult<'s, T> = Result<T, Box<dyn std::error::Error + 's>>;

pub struct Storage {
    redis: Mutex<redis::Connection>,
}

impl Storage {
    pub fn new(redis_url: &str) -> BotResult<Self> {
        let redis = redis::Client::open(redis_url)
            .and_then(|c| c.get_connection())
            .map_err(|e| format!("Redis: {}", e))?;

        Ok(Self {
            redis: Mutex::new(redis),
        })
    }

    pub fn set_add<'s, V: redis::ToRedisArgs + std::fmt::Display + Copy>(
        &'s self,
        set: &str,
        value: V,
    ) -> StorageResult<'s, ()> {
        let mut conn = self.redis.lock()?;
        conn.sadd(set, value)
            .map_err(|e| format!("Cannot add {} to {}: {}", value, set, e).into())
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

    pub fn sets_add_and_count_containing<'s, V: redis::ToRedisArgs + std::fmt::Display + Copy>(
        &'s self,
        add_to_sets: &[String],
        count_in_sets: &[String],
        value: V,
    ) -> StorageResult<'s, usize> {
        let mut pipe = redis::pipe();
        pipe.atomic();
        for set in add_to_sets {
            pipe.sadd(set, value).ignore();
        }
        for set in count_in_sets {
            pipe.sismember(set, value);
        }
        let mut conn = self.redis.lock()?;
        pipe.query::<Vec<bool>>(conn.deref_mut())
            .map(|r| r.iter().filter(|ismem| **ismem).count())
            .map_err(|e| {
                format!(
                    "Cannot add {} to sets {} with membership check across {}: {}",
                    value,
                    add_to_sets.join(","),
                    count_in_sets.join(","),
                    e
                )
                .into()
            })
    }

    pub fn sets_len<'s, S: AsRef<str>, I: IntoIterator<Item = S>>(
        &'s self,
        sets: I,
    ) -> StorageResult<'s, Vec<u64>> {
        let mut pipe = redis::pipe();
        pipe.atomic();
        for set in sets {
            pipe.scard(set.as_ref());
        }
        let mut conn = self.redis.lock()?;
        pipe.query::<Vec<u64>>(conn.deref_mut())
            .map_err(|e| format!("Cannot lookup set cardinality: {}", e).into())
    }

    pub fn hash_incr<'s, F: redis::ToRedisArgs + std::fmt::Display + Copy>(
        &'s self,
        hash: &str,
        field: F,
        delta: i64,
    ) -> StorageResult<'s, i64> {
        let mut conn = self.redis.lock()?;
        conn.hincr(hash, field, delta)
            .map_err(|e| format!("Cannot increment {}[{}] by {}: {}", hash, field, delta, e).into())
    }
}
