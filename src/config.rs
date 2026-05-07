use std::env;
use dotenv::dotenv;

pub struct Config {
    pub slack_bot_token: String,
    pub slack_channel: String,
    pub schedule_times: Vec<String>,
    pub keywords: Vec<String>,
    pub request_timeout: u64,
    pub request_delay: f64,
    pub seen_db_path: String,
}

impl Config {
    pub fn load() -> Self {
        dotenv().ok();

        let keywords = env::var("KEYWORDS")
            .unwrap_or_else(|_| "사이버,보안,개발,소프트웨어,AI,클라우드,데이터,네트워크,정보보호,플랫폼,시스템,디지털,자동화,솔루션".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let schedule_times = env::var("SCHEDULE_TIMES")
            .unwrap_or_else(|_| "09:00,14:00".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            slack_bot_token: env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN 필수"),
            slack_channel: env::var("SLACK_CHANNEL").unwrap_or_else(|_| "rnd-alerts".to_string()),
            schedule_times,
            keywords,
            request_timeout: env::var("REQUEST_TIMEOUT")
                .unwrap_or_else(|_| "15".to_string())
                .parse()
                .unwrap_or(15),
            request_delay: env::var("REQUEST_DELAY")
                .unwrap_or_else(|_| "1.5".to_string())
                .parse()
                .unwrap_or(1.5),
            seen_db_path: env::var("SEEN_DB_PATH")
                .unwrap_or_else(|_| "seen_announcements.json".to_string()),
        }
    }
}