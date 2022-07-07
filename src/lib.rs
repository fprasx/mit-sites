// TODO: Add blacklist for searches, if a search on a domaine errored, don't search it again
use log::{error, info, warn};
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display, time::Duration,
    path::Path
};

// To avoid compiling the regex multiple times in a loop
// https://docs.rs/once_cell/latest/once_cell/#lazily-compiled-regex
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

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

        let resp = match self.secure.get(base.as_str()).send().await {
            Ok(resp) => resp,
            Err(_) => {
                info!("Switching to unsecure client");
                self.unsecure.get(base.as_str()).send().await?
            }
        };

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
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to build secure client"),
            unsecure: reqwest::ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .timeout(Duration::from_secs(10))
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

    // Make sure the extension is ok
    match Path::new(url.path()).extension() {
        Some(extension) => {
            if extension != "html" 
            || extension != "htm" 
            || extension != "shtml" 
            {
                return false;
            }
        },
        // No extension means probably html
        None => ()
    }

    let str = url.as_str();

    // Look for calendar keywords, month/day/year, long numeric strings
    // avoid links with user in them
    let re = regex!(r"(?x)
    calendar|day|year|               # Avoid calendars

    (solve|kb|wikis)\.mit\.edu       # Lots of sublinks, no new links
    ");

    // Only search mit sites
    if !domain.contains(".mit.edu")
        // Can't search things like mailto or ftp
        || !url.scheme().contains("http")

        // Check a bunch of extensions, excluse some sites
        // We don't want it to match
        || re.is_match(str)
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
