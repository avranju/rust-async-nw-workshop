Summary: Implement a future by hand
Id: implement-a-future
Categories: web
Status: Published
Feedback Link: https://github.com/avranju/rust-async-nw-workshop/issues
Authors: Rajasekharan Vengalil

# Implement a future by hand

## Delay future
Duration: 1

The idea is to implement a future that does the work that Tokio's
[delay_for](https://docs.rs/tokio/0.2.11/tokio/time/fn.delay_for.html) function does.
`delay_for` takes as input a duration and returns a future that resolves when that
time duration has elapsed. Here's an [example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=6a6ac8facf27db3a5e172538697cd6e3)
of how this can be used:

```rust
use std::time::Duration;
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    println!("waiting 2 seconds");

    // wait 2 seconds
    delay_for(Duration::from_secs(2)).await;

    println!("Done waiting.");
}
```

## Create ne
Duration: 1

Boo yah