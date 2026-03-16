mod analyze;
mod cache;
mod github;
mod output;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut repo = String::new();
    let mut token = String::new();
    let mut weeks: usize = 0;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--token" => {
                i += 1;
                if i < args.len() {
                    token = args[i].clone();
                }
            }
            "--weeks" => {
                i += 1;
                if i < args.len() {
                    weeks = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("Error: --weeks must be a number, got {:?}", args[i]);
                        process::exit(1);
                    });
                }
            }
            s if s.starts_with("--") => {
                eprintln!("Error: unknown flag {:?}", s);
                process::exit(1);
            }
            s => {
                if repo.is_empty() {
                    repo = s.to_string();
                }
            }
        }
        i += 1;
    }

    if repo.is_empty() {
        eprintln!("Usage: gh-star-trend owner/repo [--token TOKEN] [--weeks N]");
        process::exit(1);
    }

    let parts: Vec<&str> = repo.splitn(2, '/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        eprintln!("Error: invalid repo format {:?}, expected owner/repo", repo);
        process::exit(1);
    }

    let owner = parts[0];
    let name = parts[1];

    if token.is_empty() {
        token = env::var("GITHUB_TOKEN").unwrap_or_default();
    }

    if token.is_empty() {
        eprintln!("Warning: no GitHub token provided. Rate limit is 60 requests/hour.");
        eprintln!("Set GITHUB_TOKEN env var or use --token flag for 5,000 requests/hour.\n");
    }

    let stars = match cache::fetch_with_cache(owner, name, &token) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    if stars.is_empty() {
        println!("No stargazers found for this repository.");
        return;
    }

    let result = analyze::analyze(&stars, weeks);
    output::print_result(&repo, &result);
}
