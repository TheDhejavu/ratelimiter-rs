use ratelimiter_rs::RateLimiter;
use std::{thread, time::Duration};

fn main() {
    let mut limiter = RateLimiter::with_in_memory();
    limiter.add_config("type1", 2, 15000).add_config("type2", 10, 30000);

    let user_id = "user12345";
    for _ in 0..10 {
        println!("RequestType1 Allowed: {}", limiter.allowed(user_id, "type1").unwrap());
        thread::sleep(Duration::from_millis(5000));
        println!("RequestType2 Allowed: {}", limiter.allowed(user_id, "type2").unwrap());
    }
}