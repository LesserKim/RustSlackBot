use scraper::{Html, Selector};
use regex::Regex;
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct KisaCrawler {
    timeout: u64,
}

impl KisaCrawler {
    pub fn new(timeout: u64) -> Self {
        Self { timeout }
    }

    fn extract_detail(&self, client: &reqwest::blocking::Client, detail_url: &str) -> std::collections::HashMap<String, String> {
        let mut extra = std::collections::HashMap::new();

        let html = match client.get(detail_url).send().and_then(|r| r.text()) {
            Ok(h) => h,
            Err(_) => return extra,
        };

        let doc = Html::parse_document(&html);

        // 테이블에서 등록마감일시 추출 (헤더 행 → 데이터 행)
        let tr_sel = Selector::parse("tr").unwrap();
        let td_sel2 = Selector::parse("td").unwrap();
        let mut deadline_col_idx: Option<usize> = None;
        for tr in doc.select(&tr_sel) {
            let tds: Vec<_> = tr.select(&td_sel2).collect();
            if deadline_col_idx.is_none() {
                for (i, td) in tds.iter().enumerate() {
                    let text = td.text().collect::<String>();
                    if text.contains("등록마감") {
                        deadline_col_idx = Some(i);
                        break;
                    }
                }
                continue;
            }
            if let Some(idx) = deadline_col_idx {
                if let Some(td) = tds.get(idx) {
                    let text = td.text().collect::<String>().trim().to_string();
                    if !text.is_empty() {
                        extra.insert("마감일".to_string(), text);
                    }
                }
                break;
            }
        }

        // 본문 텍스트에서 금액/기간 추출
        let content_sel = Selector::parse("div.board_detail_contents").unwrap();
        if let Some(content) = doc.select(&content_sel).next() {
            let text = content.text().collect::<Vec<_>>().join("\n");

            let money_patterns = [
                r"소요예산\s*:\s*([\d,]+\s*원[^\n]*)",
                r"예산액\s*:\s*([\d,]+\s*원[^\n]*)",
                r"사업비\s*:\s*([\d,]+\s*원[^\n]*)",
            ];
            for pattern in money_patterns {
                let re = Regex::new(pattern).unwrap();
                if let Some(c) = re.captures(&text) {
                    extra.insert("금액".to_string(), c.get(1).unwrap().as_str().trim().to_string());
                    break;
                }
            }

            let period_patterns = [
                r"공개기간\s*:\s*([^\n]{5,50}~[^\n]{5,50})",
                r"수행기간\s*:\s*([^\n]{5,50}~[^\n]{5,50})",
                r"사업기간\s*:\s*([^\n]{5,50}~[^\n]{5,50})",
            ];
            for pattern in period_patterns {
                let re = Regex::new(pattern).unwrap();
                if let Some(c) = re.captures(&text) {
                    extra.insert("기간".to_string(), c.get(1).unwrap().as_str().trim().to_string());
                    break;
                }
            }

            // AI 요약
            let body_text = content.text().collect::<Vec<_>>().join("\n");
            if let Some(summary) = crate::summarizer::summarize(&body_text) {
                extra.insert("AI요약".to_string(), summary);
            }
        }

        extra
    }
}

impl Crawler for KisaCrawler {
    fn source_name(&self) -> &str {
        "KISA"
    }

    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>> {
        let url = "https://www.kisa.or.kr/403?page=1&searchDiv=10&searchWord=보안";
        let client = build_client(self.timeout);
        let html = client.get(url)
            .header("Referer", "https://www.kisa.or.kr")
            .send()?
            .text()?;

        let doc = Html::parse_document(&html);
        let tbody_sel = Selector::parse("table.tbl_board.notice tbody").unwrap();
        let tr_sel = Selector::parse("tr").unwrap();
        let sbj_sel = Selector::parse("td.sbj").unwrap();
        let date_sel = Selector::parse("td.date").unwrap();
        let a_sel = Selector::parse("a").unwrap();

        let mut results = vec![];

        let tbody = match doc.select(&tbody_sel).next() {
            Some(t) => t,
            None => return Ok(results),
        };

        for tr in tbody.select(&tr_sel) {
            let td_sbj = match tr.select(&sbj_sel).next() {
                Some(t) => t,
                None => continue,
            };

            let a = match td_sbj.select(&a_sel).next() {
                Some(a) => a,
                None => continue,
            };

            let title = a.text().collect::<String>().trim().to_string();
            let href = a.value().attr("href").unwrap_or("");
            let full_url = if href.starts_with("/") {
                format!("https://www.kisa.or.kr{}", href)
            } else {
                href.to_string()
            };

            let ann_id = if full_url.contains("postSeq=") {
                full_url.split("postSeq=").nth(1).unwrap().split("&").next().unwrap().to_string()
            } else {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                full_url.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            };

            let date = tr.select(&date_sel).next()
                .map(|t| t.text().collect::<String>().trim().to_string());

            let extra = self.extract_detail(&client, &full_url);

            let mut ann = Announcement::new(
                format!("kisa_{}", ann_id),
                title,
                full_url,
                self.source_name().to_string(),
            );
            ann.date = date;
            ann.deadline = extra.get("마감일").cloned();
            ann.extra = extra;

            results.push(ann);
        }

        Ok(results)
    }
}