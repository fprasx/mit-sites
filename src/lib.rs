use log::{error, info, warn};
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
};

pub struct Seeker {
    pub found: HashSet<String>,
    pub redirects: HashSet<Redirect>,
    pub searched: HashSet<Url>,
    pub queue: VecDeque<Url>,
    secure: Client,
    unsecure: Client,
    searches: HashMap<String, usize>,
}

#[derive(PartialEq, Eq, Hash)]
pub struct Redirect {
    from: Url,
    to: Url,
}

impl Redirect {
    fn new(from: Url, to: Url) -> Self {
        Self { from, to }
    }
}

impl Display for Redirect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<{}> -> <{}>", self.from, self.to))
    }
}

impl Seeker {
    async fn get(&mut self, base: &Url) -> Result<String, Box<dyn std::error::Error>> {
        info!("Beginning search on <{base}>");

        // TODO: switch to unsecure client if need be
        let resp = self.secure.get(base.as_str()).send().await?;

        let dest = resp.url();

        // Check for if there was a redirect
        // The reason we only check if the domain changes is things like http -> https
        // or something like foo/bar -> foo/bar/
        // We might miss some edge cases but this should do the trick most of the time
        if !filter(dest) {
            warn!("{base} -> {dest}, producing an invalid link");
            return Err("redirected to invalid link".into());
        }

        if dest.domain() != base.domain() {
            let redirect = Redirect::new(base.clone(), resp.url().clone());
            warn!("    Detected redirect: {redirect}");
            self.redirects.insert(redirect);

            // If it redirected to a non-mit site, return
            if !base.as_str().contains(".mit.edu") {
                println!("{base}");
                return Err("redirected to non-mit link".into());
            }
        }

        Ok(resp.text().await?)
    }

    pub async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Get the next link
        let base = &self.queue.pop_back().ok_or("queue was empty")?;

        // Get the site
        let text = self.get(base).await?;

        // Parse html
        let html = Html::parse_document(&text);

        // Init the css selector
        let selector = Selector::parse("a").unwrap();
        let a = html.select(&selector);

        // Select the links and filter them
        let links = a
            .into_iter()
            .filter_map(|a| a.value().attr("href").and_then(|href| into_mit(base, href)))
            .filter(filter)
            .collect::<HashSet<Url>>();

        let len = links.len();
        let mut counter = 0usize;

        for link in links {
            // The domain: blank.mit.edu
            let domain = link.domain().ok_or("failed to get domain")?;

            // Book-keeping so we don't search any one site more than 200 times
            // Increase the count for searches on this domain by one
            let count = self.searches.entry(domain.into()).or_insert(0);

            // If the count is greater than 200, `continue`
            if *count > 200 {
                continue;
            }

            // Otherwise, incerement the counter
            *count += 1;

            // The URL without all the trailing bits example.mit.edu/... <- cut off ...
            self.found.insert(domain.to_owned());

            // If we haven't searched it yet at it to the queue
            if self.searched.insert(
                into_mit(base, link.as_str()).ok_or("failed to join relative link with base")?,
            ) {
                self.queue.push_back(link);
                counter += 1;
            }
        }

        let proportion = (counter as f32) / (len as f32);

        info!("Found {counter}/{len} new links ({proportion:.2})");

        Ok(())
    }

    pub fn new(roots: VecDeque<Url>) -> Self {
        Self {
            found: HashSet::new(),
            searched: HashSet::with_capacity(5000),
            redirects: HashSet::new(),
            queue: roots,
            secure: reqwest::ClientBuilder::new()
                .build()
                .expect("Failed to build secure client"),
            unsecure: reqwest::ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .build()
                .expect("Failed to build secure client"),
            searches: HashMap::new(),
        }
    }
}

// Filtering out links that are not applicable or produce bad effects
fn filter(url: &Url) -> bool {
    let domain = match url.domain() {
        Some(d) => d,
        None => return false,
    };

    let str = url.as_str();

    // Look for calendar keywords, month/day/year, long numeric strings
    // avoid links with user in them

    // Only search mit sites
    if !domain.contains(".mit.edu")
        // Can't search things like mailto or ftp
        || !url.scheme().contains("http")

        // Unhelpful extensions
        || str.contains(".pdf") 
        || str.contains(".zip") 
        || str.contains(".gz") 
        || str.contains(".mp4") 
        || str.contains(".pptx") 
        || str.contains(".32s")
        || str.contains(".JPG")
        || str.contains(".PDF")
        || str.contains(".action")
        || str.contains(".avi")
        || str.contains(".cgi")
        || str.contains(".continued")
        || str.contains(".dll")
        || str.contains(".do")
        || str.contains(".docx")
        || str.contains(".exe")
        || str.contains(".gif")
        || str.contains(".jar")
        || str.contains(".java")
        || str.contains(".jpg")
        || str.contains(".mp4")
        || str.contains(".org")
        || str.contains(".png")
        || str.contains(".pptx")
        || str.contains(".wmv")
        || str.contains(".xlsx")
        || str.contains(".com")

        // Calendars don't turn up many links, tend to cause ~infinite stays on that site
        || str.contains("calendar") 
        || str.contains("month") 
        || str.contains("day") 

        // This site has a helpdesk with a bunch of tags that have ~infinite permutations
        // TODO: maybe skip this altogether
        || (domain.contains("kb.mit.edu") && str.contains("label"))

        // This site has a helpdesk with a bunch of tags that have ~infinite permutations
        // TODO: maybe skip this altogether
        || (domain.contains("wikis.mit.edu") && str.contains("label"))

        // Lot's of sublinks, no new links
        || domain.contains("solve.mit.edu")
    {
        return false;
    }
    true
}

/// Converts an href from a page into a url
///
/// href's that are valid URLs are unchanged.
/// href's that fail to parse are relative, and are thus joined with the base
/// For example, /foo becomes base/foo
fn into_mit(base: &Url, href: &str) -> Option<Url> {
    match Url::parse(href) {
        Ok(mut url) => {
            url.set_fragment(None);
            url.set_query(None);
            Some(url)
        }
        Err(_) => match Url::join(base, href) {
            Ok(mut url) => {
                url.set_fragment(None);
                url.set_query(None);
                Some(url)
            }
            Err(e) => {
                error!("Failed to join {base} and {href}: {e}");
                None
            }
        },
    }
}
