use log::{info, warn};
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::{collections::{HashSet, VecDeque}, fmt::Display};

pub struct Seeker {
    pub found: HashSet<String>,
    pub redirects: HashSet<Redirect>,
    pub searched: HashSet<Url>,
    pub queue: VecDeque<Url>,
    secure: Client,
    unsecure: Client,
}

#[derive(PartialEq, Eq, Hash)]
pub struct Redirect {
    from: Url,
    to: Url
}

impl Redirect {
    fn new(from: Url, to: Url) -> Self {
        Self { from, to }
    }
}

impl Display for Redirect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} -> {}", self.from, self.to))
    }
}

impl Seeker {
    pub async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut base = &self.queue.pop_back().ok_or("queue was empty")?;
        info!("Beginning search on {base}");

        let resp = self.secure.get(base.as_str()).send().await?;

        // Check for if there was a redirect
        // The reason we only check if the domain changes is things like http -> https
        // or something like foo/bar -> foo/bar/
        // We might miss some edge cases but this should do the trick most of the time
        if resp.url().domain() != base.domain() {
            let redirect = Redirect::new(base.clone(), resp.url().clone());
            warn!("    Detected redirect: {redirect}");
            self.redirects.insert(redirect);
        }

        let urlclone = resp.url().clone();
        // Set base to the redirected-to URL to resolve relative paths like /foo correctly
        base = &urlclone;

        let text = resp.text().await?;

        let html = Html::parse_document(&text);

        let selector = Selector::parse("a").unwrap();
        let a = html.select(&selector);

        let links = a
            .into_iter()
            .filter_map(|a| a.value().attr("href").and_then(|href| into_mit(base, href)))
            .filter(filter)
            .collect::<HashSet<Url>>();
        info!("    Found {} links", links.len());

        for link in links {
            // The domain: blank.mit.edu
            let domain = link.domain().ok_or("failed to get domain")?;

            // The URL without all the trailing bits example.mit.edu/... <- cut off ...
            self.found.insert(domain.to_owned());

            // If we haven't searched it yet at it to the queue
            if self.searched.insert(
                into_mit(base, link.as_str()).ok_or("failed to join relative link with base")?,
            ) {
                self.queue.push_back(link);
            }
        }
        Ok(())
    }

    pub fn new(roots: VecDeque<Url>) -> Self {
        Self {
            found: HashSet::new(),
            searched: HashSet::new(),
            redirects: HashSet::new(),
            queue: roots,
            secure: reqwest::ClientBuilder::new()
                .build()
                .expect("Failed to build secure client"),
            unsecure: reqwest::ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .build()
                .expect("Failed to build secure client"),
        }
    }
}

// Filtering out links that are not applicable or produce bad effects
fn filter(url: &Url) -> bool {
    let domain = match url.domain() {
        Some(d) => d,
        None => return false
    };

    let str = url.as_str();

    // Only search mit sites
    if !domain.contains("mit.edu")
        // Can't search things like mailto or ftp
        || !url.scheme().contains("http")
        // PDF mostly likely won't contain links, stalls seeker on long lists of PDFs
        || str.contains(".pdf") 
        // This site has a calendar
        || (domain.contains("vpf.mit.edu") && str.contains("day"))
        // This site has a helpdesk with a bunch of tags that have ~infinite permutations
        // TODO: maybe skip this altogether
        || (domain.contains("kb.mit.edu") && str.contains("label"))
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
        Err(_) => {
            match Url::join(base, href) {
                Ok(mut url) => {
                    url.set_fragment(None);
                    url.set_query(None);
                    Some(url)
                }
                Err(e) => panic!("{e}"), // TODO: fix this
            }
        }
    }
}
