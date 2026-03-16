---
name: star-trend
description: Analyze GitHub repo star growth trends, star velocity, and weekly slope. Use when the user asks about a repo's star count growth, star trend, star history, popularity trajectory, growth rate, or wants to compare star momentum between repos. Also trigger when the user mentions "stars per week", "star slope", "star velocity", or asks things like "how fast is X growing" in the context of GitHub repos.
---

# GitHub Star Trend Analysis

Analyze GitHub repository star growth trends using the `gh-star-trend` CLI tool. This tool fetches stargazer timestamps from the GitHub API, groups them by week, and calculates growth metrics including simple average and linear regression slope.

## Running the tool

First, find or build the binary:

1. If `gh-star-trend` is in PATH, use it directly
2. Otherwise, look for it relative to this skill file's repo root: `./target/debug/gh-star-trend` or `./target/release/gh-star-trend`
3. If the binary doesn't exist, build it: `cargo build` in the repo root

```bash
gh-star-trend <owner/repo>
```

### Arguments

- `owner/repo` (required) — the GitHub repository
- `--weeks N` — only analyze the last N weeks
- `--token TOKEN` — GitHub personal access token (also reads `GITHUB_TOKEN` env var)

### Caching

The tool caches stargazer data in `~/.gh-star-trend/`. It remembers the last page fetched, so large repos can be fetched incrementally across multiple runs (60 pages per hour without token). No need to warn the user about this — it just works.

## Interpreting results

After running the tool, explain the output in plain language:

| Metric | Meaning |
|--------|---------|
| Avg stars/week | Total stars divided by total weeks — the baseline growth rate |
| Linear regression slope | Trend-adjusted growth rate. Higher than avg = accelerating, lower = decelerating |
| R² | How consistent the growth is. ~1.0 = steady linear growth, low = bursty/irregular |
| Last 4 weeks | Recent momentum — compare to all-time to gauge current trajectory |

Summarize with a sentence like: "This repo grows at ~X stars/week, with [accelerating/steady/slowing] momentum recently."

## Comparing repos

If the user wants to compare multiple repos, run the tool for each and present a comparison table with the key metrics side by side.
