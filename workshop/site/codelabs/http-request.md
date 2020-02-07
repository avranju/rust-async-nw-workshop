Summary: Make a HTTP request
Id: http-request
Categories: rust
Status: Published
Feedback Link: https://github.com/avranju/rust-async-nw-workshop/issues
Authors: Rajasekharan Vengalil

# Make a HTTP request

## Create a new project
Duration: 2

Create a new rust binary project like so:

```shell
cargo new http-request
```

Open `Cargo.toml` and add the following dependencies:

```toml
reqwest = { version = "0.10", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "0.2", features = ["full"] }
```

## Define a struct to de-serialize the data
Duration: 5

We will issue an HTTP `GET` request to the URL `https://httpbin.org/get`. The response will
typically be JSON that looks like this:

```json
{
    "args": {},
    "headers": {
        "Accept": "*/*",
        "Accept-Encoding": "gzip, deflate",
        "Host": "httpbin.org",
        "User-Agent": "HTTPie/0.9.8",
        "X-Amzn-Trace-Id": "Root=1-5e3ce885-d91ccb50e7e8e0c209c1338d"
    },
    "origin": "73.254.58.77",
    "url": "https://httpbin.org/get"
}
```

Define a struct that can be constructed by de-serializing the JSON given above (you can
ignore the `args` field if you like) using the `serde` and `serde-json` crates. Here's a
[short example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=7f540d05682d55b47efa921a9dd718fc)
of de-serializing structs from JSON using `serde` if you haven't tried that before:

```rust
use serde::Deserialize;
use serde_json;

#[derive(Debug, Deserialize)]
struct Person {
  name: String,
  age: i32,
}

fn main() {
  let json = r##"
  {
      "name": "Arthur Dent",
      "age": 28
  }
  "##;
  
  let p = serde_json::from_str::<Person>(json).unwrap();
  println!("{:#?}", p);
}
```

Positive
: **Note**: One way of de-serializing the `headers` field might be to use the `HashMap<String, String>` type.

## Make the request
Duration: 5

### Initialize Tokio runtime

In order to be able to run a future to completion, we need an executor which is a component
that knows how to poll a future. We use the `tokio` crate in this exercise which comes with
an efficient executor. Here's [one way](https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=acaab174b0783bc810d9f52e14b6ddb4) 
you can instantiate and run a future to completion using the tokio runtime:

```rust
use tokio;

fn main() {
  let mut runtime = tokio::runtime::Runtime::new().unwrap();
  runtime.block_on(async {
    println!("I am the future y'all!");
  });
}
```

### Handle errors

You will need to think about what the return type of `main` should be. All error types usually
implement the standard [Error](https://doc.rust-lang.org/stable/std/error/trait.Error.html)
trait so you might simply return a trait object for the `Error` trait like so:

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  let mut runtime = tokio::runtime::Runtime::new().unwrap();
  runtime.block_on(async {
    println!("I am the future y'all!");
    Ok(())
  })
}
```

Positive
: Note the absence of a `;` at the end of the last line in `main`. That should cause the return
value of `block_on` to be the return value of `main`.

### Execute the request

Use the [reqwest::get](https://docs.rs/reqwest/0.10.1/reqwest/fn.get.html) function to kick off
the request and then use the [Response::json](https://docs.rs/reqwest/0.10.1/reqwest/struct.Response.html#method.json)
function to de-serialize the JSON response into an instance of your struct.

And finally, print the result on to the console via:

```rust
println!("{:#?}", res);
```