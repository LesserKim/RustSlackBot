use scraper::{Html, Selector};
use regex::Regex;
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct KistiCrawler {
    timeout: u64,
}

impl KistiCrawler {
    pub fn new(timeout: u64) -> Self {
        Self { timeout }
    }

    fn extract_detail(&self, client: &reqwest::blocking::Client, detail_url: &str) -> (Option<String>, std::collections::HashMap<String, String>) {
        let mut extra = std::collections::HashMap::new();
        let mut deadline = None;

        // 상세 페이지 접속
        let html = match client.get(detail_url).send().and_then(|r| r.text()) {
            Ok(h) => h,
            Err(_) => return (deadline, extra),
        };

        // 접수기간 추출
        let deadline_re = Regex::new(r"접수기간\s*:\s*([^\n]+)").unwrap();
        if let Some(c) = deadline_re.captures(&html) {
            deadline = Some(c.get(1).unwrap().as_str().trim().to_string());
        }

        // 첨부파일 "바로보기" URL 찾기
        let doc = Html::parse_document(&html);
        let preview_sel = Selector::parse("div.board_file a").unwrap();
        let preview_re = Regex::new(r"filePreview\('([^']+)'").unwrap();

        // 제안요청서(RFP) 파일 우선 찾기
        for a in doc.select(&preview_sel) {
            let onclick = a.value().attr("onclick").unwrap_or("");
            if let Some(c) = preview_re.captures(onclick) {
                let filename = c.get(1).unwrap().as_str();

                // 제안요청서 파일인지 확인 (RFP, 제안요청서, 제안요청)
                let link_text = a.text().collect::<String>();
                let parent_text = a.parent().map(|p| {
                    p.children()
                    .filter_map(|c| c.value().as_text().map(|t| t.text.as_ref()))
                    .collect::<String>()
                }).unwrap_or_default();

                let combined = format!("{}{}", link_text, parent_text);
                let is_rfp = combined.contains("제안요청") || combined.contains("RFP") || combined.contains("제안 요청");

                if !is_rfp {
                    continue;
                }

                // 바로보기 HTML 접속
                let preview_url = format!(
                    "https://www.kisti.re.kr/resources/htmlconverter_skin/doc.jsp?fn={}&rs=/fatt/htmlconverter_preview",
                    filename
                );

                if let Ok(preview_html) = client.get(&preview_url).send().and_then(|r| r.text()) {
                    // 연구기간 추출
                    let period_re = Regex::new(r"연구기간\s*[:\s]*(\d{4}[\.\-]\d{2}[\.\-]?\d{0,2}\s*[~～]\s*\d{4}[\.\-]\d{2}[\.\-]?\d{0,2})").unwrap();
                    if let Some(c) = period_re.captures(&preview_html) {
                        extra.insert("기간".to_string(), c.get(1).unwrap().as_str().trim().to_string());
                    }

                    // 연구비/위탁연구비 추출
                    let money_re = Regex::new(r"(?:위탁연구비|연구비|예산)[^\d]*?([\d,]+\s*천원|[\d,]+\s*원|[\d,]+\s*백만원)").unwrap();
                    if let Some(c) = money_re.captures(&preview_html) {
                        extra.insert("금액".to_string(), c.get(1).unwrap().as_str().trim().to_string());
                    }
                }

                break;
            }
        }

        (deadline, extra)
    }
}

impl Crawler for KistiCrawler {
    fn source_name(&self) -> &str {
        "KISTI"
    }

    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>> {
        let url = "https://www.kisti.re.kr/notifications/post/research-task";
        let client = build_client(self.timeout);
        let html = client.get(url)
            .header("Referer", "https://www.kisti.re.kr")
            .send()?
            .text()?;

        let doc = Html::parse_document(&html);
        let ul_sel = Selector::parse("ul.basic_board").unwrap();
        let li_sel = Selector::parse("li").unwrap();
        let title_sel = Selector::parse("p.title a").unwrap();
        let date_sel = Selector::parse("span.date").unwrap();

        let mut results = vec![];

        let ul = match doc.select(&ul_sel).next() {
            Some(u) => u,
            None => return Ok(results),
        };

        let jsess_re = Regex::new(r";jsessionid=[^?&]*").unwrap();
        let date_re = Regex::new(r"(\d{4}[\.\-]\s*\d{2}[\.\-]\s*\d{2})").unwrap();

        for li in ul.select(&li_sel) {
            let a = match li.select(&title_sel).next() {
                Some(a) => a,
                None => continue,
            };

            let title = a.text().collect::<String>().trim().to_string();
            let href = a.value().attr("href").unwrap_or("");
            let href_clean = jsess_re.replace(href, "").to_string();

            let full_url = if href_clean.starts_with("/") {
                format!("https://www.kisti.re.kr{}", href_clean)
            } else {
                href_clean.clone()
            };

            let ann_id = full_url
                .trim_end_matches('/')
                .split('/')
                .last()
                .unwrap_or("")
                .split('?')
                .next()
                .unwrap_or("")
                .to_string();

            let ann_id = if ann_id.chars().all(|c| c.is_ascii_digit()) && !ann_id.is_empty() {
                ann_id
            } else {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                full_url.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            };

            // 등록일
            let date = li.select(&date_sel).next().and_then(|t| {
                let text = t.text().collect::<String>().trim().to_string();
                date_re.captures(&text).map(|c| c.get(1).unwrap().as_str().to_string())
            });

            // 상세 페이지 + 첨부파일에서 마감일/금액/기간 추출
            let (deadline, extra) = self.extract_detail(&client, &full_url);

            let mut ann = Announcement::new(
                format!("kisti_{}", ann_id),
                title,
                full_url,
                self.source_name().to_string(),
            );
            ann.date = date;
            ann.deadline = deadline;
            ann.extra = extra;

            results.push(ann);
        }

        Ok(results)
    }
}