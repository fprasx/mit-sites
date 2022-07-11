use seeker::Seeker;
use std::collections::VecDeque;
use url::Url;
use log::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get logging going
    env_logger::init();

    // This will be the initial start point
    let mut root = VecDeque::new();
    root.push_back(Url::parse("https://web.mit.edu").expect("failed to parse root"));

    // Initialize the seeker
    let mut seeker = Seeker::new(root);

    // Go for 20_000 cycles
    for i in 0..20_000 {
        match seeker.execute().await {
            Ok(_) => continue,
            Err(e) => error!("{i:4}: {e}"),
        }
    }

    // Display all the links found
    for i in seeker.found {
        println!("{}", i.replace(".mit.edu", ""));
    }

    Ok(())
}
