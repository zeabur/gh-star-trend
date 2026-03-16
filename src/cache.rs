use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::github;
use crate::graphql;

#[derive(Serialize, Deserialize)]
struct CacheData {
    owner: String,
    repo: String,
    updated_at: DateTime<Utc>,
    last_page: usize,
    stars: Vec<DateTime<Utc>>,
}

fn cache_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".gh-star-trend")
}

fn cache_file(owner: &str, repo: &str) -> PathBuf {
    cache_dir().join(format!("{}_{}.json", owner, repo))
}

fn load_cache(owner: &str, repo: &str) -> Option<CacheData> {
    let data = fs::read_to_string(cache_file(owner, repo)).ok()?;
    serde_json::from_str(&data).ok()
}

fn save_cache(
    owner: &str,
    repo: &str,
    stars: &[DateTime<Utc>],
    last_page: usize,
) -> Result<(), String> {
    fs::create_dir_all(cache_dir()).map_err(|e| format!("creating cache dir: {}", e))?;
    let cache = CacheData {
        owner: owner.to_string(),
        repo: repo.to_string(),
        updated_at: Utc::now(),
        last_page,
        stars: stars.to_vec(),
    };
    let data = serde_json::to_string(&cache).map_err(|e| format!("serializing cache: {}", e))?;
    fs::write(cache_file(owner, repo), data).map_err(|e| format!("writing cache: {}", e))
}

pub fn fetch_with_cache(
    owner: &str,
    repo: &str,
    token: &str,
) -> Result<Vec<DateTime<Utc>>, String> {
    let cached = load_cache(owner, repo);

    let (cached_stars, start_page) = match &cached {
        Some(c) if !c.stars.is_empty() => {
            let page = if c.last_page > 0 { c.last_page } else { 1 };
            eprintln!(
                "Cache: {} stars (last updated {}), resuming from page {}",
                c.stars.len(),
                c.updated_at.format("%Y-%m-%d %H:%M"),
                page
            );
            (c.stars.clone(), page)
        }
        _ => (Vec::new(), 1),
    };

    // Phase 1: REST API fetch
    let mut all = cached_stars.clone();
    let mut last_page = start_page;
    let mut hit_pagination_limit = false;

    match github::fetch_stargazers(owner, repo, token, &cached_stars, start_page) {
        Ok(result) => {
            all = merge_stars(&all, &result.new_stars);
            last_page = result.last_page;
            hit_pagination_limit = result.hit_pagination_limit;

            if !result.new_stars.is_empty() || result.last_page > start_page {
                eprintln!(
                    "REST: fetched {} new stars (total: {})",
                    result.new_stars.len(),
                    all.len()
                );
            } else if !cached_stars.is_empty() && !hit_pagination_limit {
                eprintln!("No new stars since last fetch.");
            }
        }
        Err(e) => {
            if e.contains("rate limit") {
                if !cached_stars.is_empty() {
                    eprintln!("Warning: rate limited, using cached data only");
                    return Ok(cached_stars);
                }
            }
            return Err(e);
        }
    }

    // Phase 2: GraphQL to fetch beyond 40k limit
    if hit_pagination_limit && !token.is_empty() {
        eprintln!("Switching to GraphQL API to fetch remaining stars...");
        match graphql::fetch_remaining_stars(owner, repo, token, &all) {
            Ok(gql_result) => {
                if !gql_result.new_stars.is_empty() {
                    all = merge_stars(&all, &gql_result.new_stars);
                    eprintln!(
                        "GraphQL: fetched {} additional stars (total: {}, actual: {})",
                        gql_result.new_stars.len(),
                        all.len(),
                        gql_result.total_count
                    );
                }
            }
            Err(e) => {
                eprintln!("Warning: GraphQL fetch failed: {}. Using REST data only.", e);
            }
        }
    }

    // Save cache
    if all.len() > cached_stars.len() {
        if let Err(e) = save_cache(owner, repo, &all, last_page) {
            eprintln!("Warning: failed to save cache: {}", e);
        }
    }

    Ok(all)
}

fn merge_stars(cached: &[DateTime<Utc>], new_stars: &[DateTime<Utc>]) -> Vec<DateTime<Utc>> {
    if cached.is_empty() {
        let mut v = new_stars.to_vec();
        v.sort();
        return v;
    }
    if new_stars.is_empty() {
        return cached.to_vec();
    }

    let seen: HashSet<i64> = cached.iter().map(|t| t.timestamp()).collect();
    let mut merged = cached.to_vec();
    for t in new_stars {
        if !seen.contains(&t.timestamp()) {
            merged.push(*t);
        }
    }
    merged.sort();
    merged
}
