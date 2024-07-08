// Rate limiter using sliding window technique

use std::{collections::HashMap, error::Error};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::error::RateLimiterError;
use crate::storage::Storage;
use std::sync::Arc;
pub struct RateLimiter {
    configs: HashMap<String, Config>,
    storage: Storage,
}

#[derive(Debug)]
struct Config {
    capacity: u32,
    window_time: Duration,
}

impl RateLimiter {
    /// Creates a new rate limiter with in-memory storage.
    pub fn with_in_memory() -> Self {
        Self {
            configs: HashMap::new(),
            storage: Storage::InMemory(Arc::new(Mutex::new(HashMap::new()))),
        }
    }
    /// Creates a new rate limiter with Redis storage.
    ///
    /// # Arguments
    ///
    /// * `redis_url` - The URL of the Redis server.
    pub fn with_redis(redis_url: &str) -> Self {
        let client = redis::Client::open(redis_url).unwrap();
        Self {
            configs: HashMap::new(),
            storage: Storage::Redis(client),
        }
    }

    /// Adds a configuration for a request type.
    ///
    /// # Arguments
    ///
    /// * `request_type` - The type of request to configure.
    /// * `capacity` - The maximum number of requests allowed in the window time.
    /// * `window_time_millis` - The length of the sliding window in milliseconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratelimiter_rs::RateLimiter;
    /// 
    /// let mut limiter = RateLimiter::with_in_memory();
    /// limiter.add_config("type1", 5, 60000);
    /// ```
    pub fn add_config(&mut self, request_type: &str, capacity: u32, window_time_millis: u64) -> &mut Self {
        self.configs.insert(
            request_type.to_string(),
            Config {
                capacity,
                window_time: Duration::from_millis(window_time_millis),
            },
        );
        self
    }

    /// Checks if a request is allowed.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user making the request.
    /// * `request_type` - The type of request.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the request is allowed.
    /// * `Ok(false)` if the request is not allowed.
    /// * `Err` if an error occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratelimiter_rs::RateLimiter;
    /// 
    /// let mut limiter = RateLimiter::with_in_memory();
    /// let is_allowed = limiter.allowed("user1", "type1").unwrap();
    /// ```
    pub fn allowed(&self, user_id: &str, request_type: &str) -> Result<bool, Box<dyn Error>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        let config = match self.configs.get(request_type) {
            Some(config) => config,
            None => return Ok(false),
        };

        let start_time_in_millis = now - config.window_time.as_millis() as u64;
        let end_time_in_millis = now;
        let eviction_time_in_millis = now - config.window_time.as_millis() as u64;

        match &self.storage {
            Storage::InMemory(storage) => {
                let mut storage = storage.lock().map_err(|_| RateLimiterError::Message("unable to acquire lock".to_string()))?;
                let user_request_logs = storage.entry(user_id.to_string()).or_insert_with(Vec::new);

                // evict expired entries by retaining timestamp greater than the eviction time.
                user_request_logs.retain(|&timestamp| timestamp >= eviction_time_in_millis);
                
                // count number of requests in the last window
                let request_count = user_request_logs.iter().filter(|&&timestamp| timestamp <= end_time_in_millis).count();

                if request_count < config.capacity as usize {
                    user_request_logs.push(now);
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            Storage::Redis(client) => {
                let mut con = client.get_connection()?;
                // Reference: https://engineering.grab.com/frequency-capping
                let script = redis::Script::new(r"
                    local user_redis_key = KEYS[1]
                    local limit_value = tonumber(ARGV[1])
                    local start_time_in_millis = tonumber(ARGV[2])
                    local end_time_in_millis = tonumber(ARGV[3])
                    local current_time_in_millis = tonumber(ARGV[4])
                    local eviction_time_in_millis = tonumber(ARGV[5])

                    local request_count = redis.call('ZCOUNT', user_redis_key, start_time_in_millis, end_time_in_millis)

                    if tonumber(request_count) < limit_value then
                        redis.call('ZADD', user_redis_key, current_time_in_millis, current_time_in_millis)
                        redis.call('ZREMRANGEBYSCORE', user_redis_key, '-inf', eviction_time_in_millis)
                        return 1
                    else
                        return 0
                    end
                ");

                let key = format!("{}:{}", user_id, request_type);
                let result: i32 = script.arg(config.capacity)
                                        .arg(start_time_in_millis)
                                        .arg(end_time_in_millis)
                                        .arg(now)
                                        .arg(eviction_time_in_millis)
                                        .key(key)
                                        .invoke(&mut con)?;
                if result == 1 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
        }
    }

}


#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};
    use super::*;

    #[test]
    fn test_with_in_memory(){
        let mut limiter = RateLimiter::with_in_memory();
        limiter.add_config("type1", 2, 5000).add_config("type2", 10, 30000);

        let user_id = "user12345";
        for _ in 0..2 {
            let is_allowed = limiter.allowed(user_id, "type1").unwrap();
            thread::sleep(Duration::from_millis(1000));
            assert!(is_allowed);
        }

        // Previous request was two and maximum capacity is 2 within 2s with 5s window. Below we ensure that request 
        // are no longer allowed because 5s overlaps and there is no enough capacity to handle new requests.
        for _ in 0..2 {
            let is_allowed = limiter.allowed(user_id, "type1").unwrap();
            assert_eq!(is_allowed, false);
            thread::sleep(Duration::from_millis(1000));
        }
    }

}