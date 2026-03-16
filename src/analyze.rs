use chrono::{DateTime, Datelike, Utc};
use std::collections::BTreeMap;

pub struct WeekData {
    pub year: i32,
    pub week: u32,
    pub count: usize,
}

pub struct AnalysisResult {
    pub total_stars: usize,
    pub first_star: DateTime<Utc>,
    pub last_star: DateTime<Utc>,
    pub total_weeks: usize,

    pub avg_per_week: f64,
    pub slope: f64,
    pub r_squared: f64,
    pub recent_avg: f64,
    pub recent_slope: f64,
    pub recent_r_squared: f64,
    pub recent_weeks_n: usize,

    pub weekly_breakdown: Vec<WeekData>,
}

pub fn analyze(stars: &[DateTime<Utc>], limit_weeks: usize) -> AnalysisResult {
    let mut sorted = stars.to_vec();
    sorted.sort();

    // Group by ISO week
    let mut week_map: BTreeMap<(i32, u32), usize> = BTreeMap::new();
    for t in &sorted {
        let key = (t.iso_week().year(), t.iso_week().week());
        *week_map.entry(key).or_default() += 1;
    }

    let mut weeks: Vec<WeekData> = week_map
        .into_iter()
        .map(|((year, week), count)| WeekData { year, week, count })
        .collect();

    // If --weeks flag used, limit
    if limit_weeks > 0 && weeks.len() > limit_weeks {
        weeks = weeks.split_off(weeks.len() - limit_weeks);
    }

    let total_stars = sorted.len();

    let mut result = AnalysisResult {
        total_stars,
        first_star: sorted[0],
        last_star: sorted[sorted.len() - 1],
        total_weeks: weeks.len(),
        avg_per_week: 0.0,
        slope: 0.0,
        r_squared: 0.0,
        recent_avg: 0.0,
        recent_slope: 0.0,
        recent_r_squared: 0.0,
        recent_weeks_n: 0,
        weekly_breakdown: Vec::new(),
    };

    if weeks.is_empty() {
        return result;
    }

    // Build cumulative series
    let mut cumulative = Vec::with_capacity(weeks.len());
    let mut sum = 0.0;
    for w in &weeks {
        sum += w.count as f64;
        cumulative.push(sum);
    }

    result.avg_per_week = sum / weeks.len() as f64;
    let (slope, r2) = linear_regression(&cumulative);
    result.slope = slope;
    result.r_squared = r2;

    // Recent trend (last 4 weeks)
    let recent_n = weeks.len().min(4);
    result.recent_weeks_n = recent_n;

    if recent_n > 1 {
        let recent_weeks = &weeks[weeks.len() - recent_n..];
        let mut recent_cum = Vec::with_capacity(recent_n);
        let mut rsum = 0.0;
        for w in recent_weeks {
            rsum += w.count as f64;
            recent_cum.push(rsum);
        }
        result.recent_avg = rsum / recent_n as f64;
        let (rs, rr) = linear_regression(&recent_cum);
        result.recent_slope = rs;
        result.recent_r_squared = rr;
    } else if recent_n == 1 {
        result.recent_avg = weeks.last().unwrap().count as f64;
    }

    result.weekly_breakdown = weeks;
    result
}

fn linear_regression(y: &[f64]) -> (f64, f64) {
    let n = y.len() as f64;
    if n < 2.0 {
        return (0.0, 0.0);
    }

    let sum_x: f64 = (0..y.len()).map(|i| i as f64).sum();
    let sum_y: f64 = y.iter().sum();
    let mean_x = sum_x / n;
    let mean_y = sum_y / n;

    let mut num = 0.0;
    let mut den = 0.0;
    for (i, v) in y.iter().enumerate() {
        let dx = i as f64 - mean_x;
        let dy = v - mean_y;
        num += dx * dy;
        den += dx * dx;
    }

    if den == 0.0 {
        return (0.0, 0.0);
    }

    let slope = num / den;
    let intercept = mean_y - slope * mean_x;

    let mut ss_res = 0.0;
    let mut ss_tot = 0.0;
    for (i, v) in y.iter().enumerate() {
        let predicted = slope * i as f64 + intercept;
        ss_res += (v - predicted).powi(2);
        ss_tot += (v - mean_y).powi(2);
    }

    let r_squared = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        0.0
    };

    (slope, r_squared)
}
