use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashSet;
use std::io::{self, Write};

#[derive(Deserialize)]
struct GraphQLResponse {
    data: Option<GraphQLData>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Deserialize)]
struct GraphQLData {
    repository: Option<RepoData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepoData {
    stargazer_count: usize,
    stargazers: StargazerConnection,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StargazerConnection {
    edges: Vec<StargazerEdge>,
    page_info: PageInfo,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_previous_page: bool,
    start_cursor: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StargazerEdge {
    starred_at: DateTime<Utc>,
}

pub struct GraphQLResult {
    pub total_count: usize,
    pub new_stars: Vec<DateTime<Utc>>,
}

/// Fetch remaining stars via GraphQL, going backwards from newest.
/// Stops when it finds stars that overlap with known_stars.
pub fn fetch_remaining_stars(
    owner: &str,
    repo: &str,
    token: &str,
    known_stars: &[DateTime<Utc>],
) -> Result<GraphQLResult, String> {
    if token.is_empty() {
        return Err("GraphQL API requires a GitHub token".to_string());
    }

    let known: HashSet<i64> = known_stars.iter().map(|t| t.timestamp()).collect();
    let mut new_stars: Vec<DateTime<Utc>> = Vec::new();
    let mut cursor: Option<String> = None;
    let mut total_count = 0;
    let mut page = 0;

    loop {
        page += 1;

        let before_clause = match &cursor {
            Some(c) => format!(", before: \"{}\"", c),
            None => String::new(),
        };

        let query = format!(
            r#"query {{ repository(owner: "{}", name: "{}") {{ stargazerCount stargazers(last: 100{}) {{ edges {{ starredAt }} pageInfo {{ hasPreviousPage startCursor }} }} }} }}"#,
            owner, repo, before_clause
        );

        let body = serde_json::json!({ "query": query }).to_string();

        let resp = ureq::post("https://api.github.com/graphql")
            .header("Authorization", &format!("Bearer {}", token))
            .header("User-Agent", "gh-star-trend")
            .header("Content-Type", "application/json")
            .send(&body)
            .map_err(|e| format!("GraphQL request failed: {}", e))?;

        let result: GraphQLResponse = resp
            .into_body()
            .read_json()
            .map_err(|e| format!("parsing GraphQL response: {}", e))?;

        if let Some(errors) = result.errors {
            let msgs: Vec<String> = errors.into_iter().map(|e| e.message).collect();
            return Err(format!("GraphQL errors: {}", msgs.join(", ")));
        }

        let data = result.data.ok_or("no data in GraphQL response")?;
        let repo_data = data.repository.ok_or("repository not found")?;

        total_count = repo_data.stargazer_count;
        let connection = repo_data.stargazers;

        let mut found_overlap = false;
        for edge in &connection.edges {
            if known.contains(&edge.starred_at.timestamp()) {
                found_overlap = true;
            } else {
                new_stars.push(edge.starred_at);
            }
        }

        let fetched_total = new_stars.len();
        let remaining = if total_count > known_stars.len() + fetched_total {
            total_count - known_stars.len() - fetched_total
        } else {
            0
        };

        eprint!(
            "\rGraphQL: fetched {} new stars (page {}, ~{} remaining)",
            fetched_total, page, remaining
        );
        io::stderr().flush().ok();

        // Stop if we found overlap with cached data or no more pages
        if found_overlap || !connection.page_info.has_previous_page {
            break;
        }

        cursor = connection.page_info.start_cursor;
    }

    if !new_stars.is_empty() || page > 1 {
        eprintln!();
    }

    Ok(GraphQLResult {
        total_count,
        new_stars,
    })
}
