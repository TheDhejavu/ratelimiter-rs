# ratelimiter-rs
A Rust implementation of Sliding Window Algorithm for distributed rate limiting.

>  ⚠️ this is an experiment 


![sliding window](https://github.com/TheDhejavu/ratelimiter-rs/blob/main/docs/sliding-window.png)


## Installation

Add `ratelimiter-rs` to the `[dependencies]` section of your `Cargo.toml`:

```toml
...

[dependencies]
ratelimiter-rs = { git =  "https://github.com/TheDhejavu/ratelimiter-rs.git" }

...
```

## Usage

`src/main.rs`:

```rust
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
```

## References
- [Frequency Capping](https://engineering.grab.com/frequency-capping)
