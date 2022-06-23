use std::collections::HashSet;

use scraper::{selector::Selector, Html};
use url::Url;
// For later if we multithread
// use crossbeam_queue::SegQueue;

pub const RESET: &str = "\x1B[0m";
pub const RED: &str = "\x1B[0;31m"; // Red
pub const GREEN: &str = "\x1B[0;32m"; // Green

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base = Url::parse("https://mit.edu").unwrap();
    let text = reqwest::get(base.as_str()).await?.text().await?;
    let html = Html::parse_document(&text);
    let selector = Selector::parse("a").unwrap();
    let a = html.select(&selector);
    let l = a
        .into_iter()
        .filter_map(|a| {
            a.value()
                .attr("href")
                .and_then(|href| match Url::parse(href) {
                    Ok(url) => Some(url),
                    Err(_) => match Url::join(&base, href) {
                        Ok(url) => Some(url),
                        Err(e) => {
                            println!("Couldn't parse {href}, error:\n\t{e}");
                            None
                        }
                    },
                })
        })
        .filter(|url| url.domain().unwrap_or("").contains(".mit.edu"))
        .collect::<Vec<Url>>();

    for i in l.iter() {
        println!("{i}")
    }
    Ok(())
}

enum MyOption<T> {
    Some(T),
    None,
}

struct SearchResults<'a> {
    link: &'a str,
    depth: usize,
    max: usize,
    found: HashSet<Url>,
}
