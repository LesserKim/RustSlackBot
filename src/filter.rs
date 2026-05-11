use chrono::{Local, NaiveDate};
use regex::Regex;
use crate::models::Announcement;

pub fn parse_deadline(s: &str) -> Option<NaiveDate> {
    if s.is_empty() {
        return None;
    }

    // 특수 케이스
    let specials = ["상시", "별도", "미정", "추후"];
    if specials.iter().any(|sp| s.contains(sp)) {
        return None;
    }

    // 숫자만 추출
    let re = Regex::new(r"\d").unwrap();
    let digits: String = re.find_iter(s).map(|m| m.as_str()).collect();

    if digits.len() >= 8 {
        // YYYYMMDD
        NaiveDate::parse_from_str(&digits[..8], "%Y%m%d").ok()
    } else if digits.len() >= 6 {
        // YYMMDD
        NaiveDate::parse_from_str(&digits[..6], "%y%m%d").ok()
    } else {
        None
    }
}

pub fn is_not_expired(ann: &Announcement) -> bool {
    let deadline = match &ann.deadline {
        Some(d) if !d.is_empty() => d,
        _ => return true,
    };

    match parse_deadline(deadline) {
        Some(d) => d >= Local::now().date_naive(),
        None => true,  // 파싱 실패하면 일단 통과
    }
}

pub fn match_keywords(ann: &Announcement, keywords: &[String]) -> Vec<String> {
    let title = ann.title.to_lowercase();
    keywords
        .iter()
        .filter(|kw| title.contains(&kw.to_lowercase()))
        .cloned()
        .collect()
}

pub fn filter_announcements(
    announcements: Vec<Announcement>,
    keywords: &[String],
) -> Vec<Announcement> {
    announcements
        .into_iter()
        .filter(is_not_expired)
        .filter_map(|mut ann| {
            let matched = match_keywords(&ann, keywords);
            if matched.is_empty() {
                None
            } else {
                ann.matched_keywords = matched;
                Some(ann)
            }
        })
        .collect()
}