use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashSet;
use std::io::{self, Write};

#[derive(Deserialize)]
struct StargazerResponse {
    starred_at: DateTime<Utc>,
}

pub struct FetchResult {
    pub new_stars: Vec<DateTime<Utc>>,
    pub last_page: usize,
}

pub fn fetch_stargazers(
    owner: &str,
    repo: &str,
    token: &str,
    known_stars: &[DateTime<Utc>],
    start_page: usize,
) -> Result<FetchResult, String> {
    let known: HashSet<i64> = known_stars.iter().map(|t| t.timestamp()).collect();
    let mut new_stars = Vec::new();
    let mut page = start_page;
    let mut total_pages = 0;

    loop {
        let url = format!(
            "https://api.github.com/repos/{}/{}/stargazers?per_page=100&page={}",
            owner, repo, page
        );

        let mut req = ureq::get(&url)
            .header("Accept", "application/vnd.github.v3.star+json")
            .header("User-Agent", "gh-star-trend");

        if !token.is_empty() {
            req = req.header("Authorization", &format!("Bearer {}", token));
        }

        let resp = req.call().map_err(|e| {
            if let ureq::Error::StatusCode(404) = e {
                return format!("repository {}/{} not found", owner, repo);
            }
            if let ureq::Error::StatusCode(403) = e {
                return "rate limit exceeded".to_string();
            }
            format!("fetching stargazers: {}", e)
        })?;

        if total_pages == 0 {
            if let Some(link) = resp.headers().get("link") {
                if let Ok(link_str) = link.to_str() {
                    total_pages = parse_total_pages(link_str);
                }
            }
        }

        let body = resp
            .into_body()
            .read_to_string()
            .map_err(|e| format!("reading response: {}", e))?;

        let stars: Vec<StargazerResponse> =
            serde_json::from_str(&body).map_err(|e| format!("parsing response: {}", e))?;

        let page_len = stars.len();

        for s in &stars {
            if !known.contains(&s.starred_at.timestamp()) {
                new_stars.push(s.starred_at);
            }
        }

        if total_pages > 0 {
            eprint!(
                "\rFetching stargazers... page {}/{} ({} new stars)",
                page,
                total_pages,
                new_stars.len()
            );
        } else {
            eprint!(
                "\rFetching stargazers... page {} ({} new stars)",
                page,
                new_stars.len()
            );
        }
        io::stderr().flush().ok();

        if page_len < 100 {
            break;
        }
        page += 1;
    }

    if page > start_page || !new_stars.is_empty() {
        eprintln!();
    }

    Ok(FetchResult {
        new_stars,
        last_page: page,
    })
}

fn parse_total_pages(link_header: &str) -> usize {
    for part in link_header.split(',') {
        if part.contains("rel=\"last\"") {
            // Extract page=N from the URL
            if let Some(start) = part.find("page=") {
                let rest = &part[start + 5..];
                if let Some(end) = rest.find(|c: char| !c.is_ascii_digit()) {
                    if let Ok(n) = rest[..end].parse() {
                        return n;
                    }
                }
            }
        }
    }
    0
}
