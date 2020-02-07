use std::collections::HashMap;
use std::error::Error;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct HttpBin {
    headers: HashMap<String, String>,
    origin: String,
    url: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let resp = reqwest::get("https://httpbin.org/get")
            .await?
            .json::<HttpBin>()
            .await?;

        println!("{:#?}", resp);
        Ok(())
    })
}
