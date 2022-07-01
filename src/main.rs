use seeker::Seeker;
use std::collections::VecDeque;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get loggin going
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
            Err(e) => println!("{i:4}: {e}"),
        }
    }

    // Display all the links found
    println!("Found:");
    for i in seeker.found {
        println!("{i}");
    }

    // Print the number of links searched
    println!("{:#?}", seeker.searched.len());

    // Print the number of links in the queue
    println!("On the docket: {:#?}", seeker.queue.len());

    Ok(())
}
