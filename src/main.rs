
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://mit.edu")
        .await?
        .text()
        .await?;
    println!("{resp:#?}");
    Ok(())
}
