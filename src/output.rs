use crate::analyze::AnalysisResult;

pub fn print_result(repo: &str, r: &AnalysisResult) {
    println!("Repository: {}", repo);
    println!("Total stars: {}", format_number(r.total_stars));
    println!(
        "Tracked period: {} ~ {} ({} weeks)",
        r.first_star.format("%Y-%m-%d"),
        r.last_star.format("%Y-%m-%d"),
        r.total_weeks,
    );

    println!();
    println!("=== Growth Analysis ===");
    println!("Avg stars/week (all time):    {:.1}", r.avg_per_week);
    println!(
        "Linear regression slope:      {:.1} stars/week (R²={:.2})",
        r.slope, r.r_squared
    );

    if r.recent_weeks_n > 1 {
        println!(
            "Avg stars/week (last {} wks):  {:.1}",
            r.recent_weeks_n, r.recent_avg
        );
        println!(
            "Linear regression (last {}):   {:.1} stars/week (R²={:.2})",
            r.recent_weeks_n, r.recent_slope, r.recent_r_squared
        );
    }

    // Weekly breakdown (last 10 weeks)
    println!();
    let weeks = &r.weekly_breakdown;
    let start = if weeks.len() > 10 {
        weeks.len() - 10
    } else {
        0
    };
    let show = &weeks[start..];
    println!("=== Weekly Breakdown (last {} weeks) ===", show.len());
    for w in show {
        println!("{}-W{:02}:  +{}", w.year, w.week, w.count);
    }
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    let remainder = bytes.len() % 3;

    if remainder > 0 {
        result.push_str(&s[..remainder]);
    }
    for i in (remainder..bytes.len()).step_by(3) {
        if !result.is_empty() {
            result.push(',');
        }
        result.push_str(&s[i..i + 3]);
    }
    result
}
