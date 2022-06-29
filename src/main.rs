use seeker::Seeker;
use std::collections::VecDeque;
use url::Url;

pub const RESET: &str = "\x1B[0m";
pub const RED: &str = "\x1B[0;31m"; // Red
pub const GREEN: &str = "\x1B[0;32m"; // Green

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut root = VecDeque::new();
    root.push_back(Url::parse("https://mit.edu").expect("failed to parse root"));
    let mut seeker = Seeker::new(root);
    for i in 0..10_000 {
        match seeker.execute().await {
            Ok(_) => continue,
            Err(e) => println!("{i:4}: {e}"),
        }
    }
    println!("Found:");
    for i in seeker.found {
        println!("{i}");
    }
    println!("{:#?}", seeker.searched.len());
    println!("On the docket: {:#?}", seeker.queue.len());
    Ok(())
}
