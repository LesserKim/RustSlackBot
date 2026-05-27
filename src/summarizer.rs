use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

pub fn summarize(text: &str) -> Option<String>
{
    let client = Client::builder()
    .timeout(Duration::from_secs(120))
    .build()
    .ok()?;

    let prompt = format!(
        "아래 입찰공고문을 읽고 핵심만 요약해줘.금액, 기간, 자격요건, 일정, 유의사항 중심으로 표로 정리해줘. 반드시 아래 형식으로만 답변 필요.:\n\
        - 예산: (금액)\n\
        - 기간: (수행기간)\n\
        - 핵심내용: (요약)\n\
        - 자격: (참여자격)\n\
        정보가 없다면 해당 항목은 생략할 것.\n\n\
        공고 본문:\n{}"
        &text[..text.len().min(3000)]
    );

    let body = json!({
        "model":"qwen2.5:7b",
        "prompt":prompt,
        "stream":false
    });

    let resp = client
    .post("http://localhost:11434/api/generate")
    .json(&body)
    .send()
    .ok()?;

    let result: serde_json::Value = resp.json().ok()?;
    result["response"].as_str().map(|s| s.trim(),to_string())
}