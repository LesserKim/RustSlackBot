use std::collections::HashMap;
use scraper::{Html, Selector};
use regex::Regex;
use super::crawlers::base::build_client;

pub fn fetch_detail(url: &str, content_selector: &str, timeout: u64) -> HashMap<String, String> {
    let mut result = HashMap::new();

    let client = build_client(timeout);
    let html = match client.get(url).send().and_then(|r| r.text()) {
        Ok(h) => h,
        Err(_) => return result,
    };

    let doc = Html::parse_document(&html);
    let sel = match Selector::parse(content_selector) {
        Ok(s) => s,
        Err(_) => return result,
    };

    let content = match doc.select(&sel).next() {
        Some(c) => c,
        None => return result,
    };

    // 테이블에서 마감일 추출
    let th_sel = Selector::parse("th, td").unwrap();
    let mut prev_was_deadline_label = false;
    for cell in content.select(&th_sel) {
        let text = cell.text().collect::<String>();
        let text = text.trim();
        if prev_was_deadline_label {
            result.insert("마감일".to_string(), text.to_string());
            prev_was_deadline_label = false;
        }
        if text.contains("등록마감") || text.contains("마감일시") {
            prev_was_deadline_label = true;
        }
    }

    // 전체 텍스트
    let text = content.text().collect::<Vec<_>>().join("\n");

    // 금액 패턴
    let money_patterns = [
        r"소요예산\s*:\s*([\d,]+원[^\n]*)",
        r"예산액\s*:\s*([\d,]+원[^\n]*)",
        r"사업비\s*:\s*([\d,]+원[^\n]*)",
        r"예산\s*:\s*([\d,]+원[^\n]*)",
        r"([\d,]+원)\s*\(부가세\s*포함\)",
    ];
    for pattern in money_patterns {
        let re = Regex::new(pattern).unwrap();
        if let Some(c) = re.captures(&text) {
            result.insert("금액".to_string(), c.get(1).unwrap().as_str().trim().to_string());
            break;
        }
    }

    // 기간 패턴
    let period_patterns = [
        r"공개기간\s*:\s*([^\n]{5,50}~[^\n]{5,50})",
        r"수행기간\s*:\s*([^\n]{5,50}~[^\n]{5,50})",
        r"사업기간\s*:\s*([^\n]{5,50}~[^\n]{5,50})",
        r"계약기간\s*:\s*([^\n]{5,50}~[^\n]{5,50})",
    ];
    for pattern in period_patterns {
        let re = Regex::new(pattern).unwrap();
        if let Some(c) = re.captures(&text) {
            result.insert("기간".to_string(), c.get(1).unwrap().as_str().trim().to_string());
            break;
        }
    }

    // 자격 패턴
    let qual_patterns = [
        r"신청자격\s*:\s*([^\n]{10,150})",
        r"지원자격\s*:\s*([^\n]{10,150})",
        r"참여자격\s*:\s*([^\n]{10,150})",
        r"입찰참가자격\s*:\s*([^\n]{10,150})",
    ];
    for pattern in qual_patterns {
        let re = Regex::new(pattern).unwrap();
        if let Some(c) = re.captures(&text) {
            result.insert("자격".to_string(), c.get(1).unwrap().as_str().trim().to_string());
            break;
        }
    }

    result
}