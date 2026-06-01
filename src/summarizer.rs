use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

pub fn summarize(text: &str) -> Option<String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .ok()?;

    let truncated: String = text.chars().take(1000).collect();

    let prompt = format!(
        "다음 R&D 공고 본문을 간결하게 요약해줘. 반드시 아래 형식으로만 답변해:\n\
        - 예산: (금액)\n\
        - 기간: (수행기간)\n\
        - 핵심내용: (한 줄 요약)\n\
        - 자격: (참여자격)\n\
        정보가 없으면 해당 항목은 생략해.\n\n\
        공고 본문:\n{}",
        truncated
    );

    let body = json!({
        "model": "qwen2.5:0.5b",
        "prompt": prompt,
        "stream": false
    });

    let resp = client
        .post("http://localhost:11434/api/generate")
        .json(&body)
        .send()
        .ok()?;

    let result: serde_json::Value = resp.json().ok()?;
    result["response"].as_str().map(|s| s.trim().to_string())
}