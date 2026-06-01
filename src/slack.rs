use std::time::Duration;
use serde_json::{json, Value};
use reqwest::blocking::Client;
use crate::models::Announcement;

pub struct SlackNotifier {
    token: String,
    channel: String,
    client: Client,
}

impl SlackNotifier {
    pub fn new(token: String, channel: String) -> Self {
        Self {
            token,
            channel,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("HTTP 클라이언트 생성 실패"),
        }
    }

    fn build_blocks(&self, anns: &[&Announcement]) -> Vec<Value> {
        let mut blocks = vec![
            json!({
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": format!("R&D 공고 알림 - {}건", anns.len()),
                    "emoji": true,
                }
            }),
            json!({ "type": "divider" }),
        ];

        for ann in anns {
            let keyword_str = ann.matched_keywords.join(" / ");

            let mut meta_parts = vec![];
            meta_parts.push(format!("출처: {}", ann.source));
            if let Some(a) = &ann.agency { meta_parts.push(format!("기관: {}", a)); }
            if let Some(d) = &ann.date { meta_parts.push(format!("등록: {}", d)); }
            if let Some(d) = &ann.deadline { meta_parts.push(format!("마감: {}", d)); }
            let meta_str = meta_parts.join(" | ");

            let mut extra_parts = vec![];
            if let Some(v) = ann.extra.get("마감일") { extra_parts.push(format!("마감: {}", v)); }
            if let Some(v) = ann.extra.get("금액") { extra_parts.push(format!("금액: {}", v)); }
            if let Some(v) = ann.extra.get("기간") { extra_parts.push(format!("기간: {}", v)); }
            if let Some(v) = ann.extra.get("자격") { extra_parts.push(format!("자격: {}", v)); }
            let extra_str = if extra_parts.is_empty() {
                String::new()
            } else {
                format!("\n{}", extra_parts.join(" | "))
            };

            let ai_str = if let Some(v) = ann.extra.get("AI요약") {
                format!("\n📋 {}", v)
            } else {
                String::new()
            };

            blocks.push(json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!(
                        "*<{}|{}>*\n{}\n키워드: {}{}{}",
                        ann.url, ann.title, meta_str, keyword_str, extra_str, ai_str
                    )
                }
            }));
            blocks.push(json!({ "type": "divider" }));
        }

        blocks
    }

    pub fn send(&self, announcements: &[&Announcement]) -> bool {
        if announcements.is_empty() {
            return true;
        }

        let batch_size = 15;
        for chunk in announcements.chunks(batch_size) {
            let payload = json!({
                "channel": self.channel,
                "blocks": self.build_blocks(chunk),
                "text": format!("R&D 공고 {}건", chunk.len()),
                "unfurl_links": false,
                "unfurl_media": false,
            });

            let resp = self.client
                .post("https://slack.com/api/chat.postMessage")
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Content-Type", "application/json; charset=utf-8")
                .body(serde_json::to_vec(&payload).unwrap())
                .send();

            match resp {
                Ok(r) => {
                    let result: Value = r.json().unwrap_or(json!({}));
                    if !result["ok"].as_bool().unwrap_or(false) {
                        log::error!("슬랙 전송 실패: {}", result["error"]);
                        return false;
                    }
                    log::info!("슬랙 전송 완료: {}건", chunk.len());
                }
                Err(e) => {
                    log::error!("슬랙 요청 실패: {}", e);
                    return false;
                }
            }
        }

        true
    }
}