use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: rtalk-client <user_name>");
        return Ok(());
    }
    let _user_name = args[1].clone();

    // Implement the client.

    Ok(())
}
