use std::time::Duration;
use reqwest::blocking::Client;
use crate::models::Announcement;

pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

pub fn build_client(timeout_secs: u64) -> Client {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(USER_AGENT)
        .build()
        .expect("HTTP 클라이언트 생성 실패")
}

pub trait Crawler {
    fn source_name(&self) -> &str;
    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>>;

    fn safe_fetch(&self) -> Vec<Announcement> {
        match self.fetch() {
            Ok(results) => {
                log::info!("[{}] {}건 수집", self.source_name(), results.len());
                results
            }
            Err(e) => {
                log::error!("[{}] 크롤링 실패: {}", self.source_name(), e);
                vec![]
            }
        }
    }
}