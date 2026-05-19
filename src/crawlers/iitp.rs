use scraper::{Html, Selector};
use regex::Regex;
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct IitpCrawler {
    timeout: u64,
}

impl IitpCrawler {
    pub fn new(timeout: u64) -> Self {
        Self { timeout }
    }
}

impl Crawler for IitpCrawler {
    fn source_name(&self) -> &str {
        "IITP"
    }

    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>> {
        let url = "https://ezone.iitp.kr/common/anno/list";
        let client = build_client(self.timeout);
        let html = client.get(url)
            .header("Referer", "https://ezone.iitp.kr")
            .send()?
            .text()?;

        let doc = Html::parse_document(&html);
        let li_sel = Selector::parse("li.clearfix").unwrap();
        let tit_a_sel = Selector::parse("strong.bbs_tit a[onclick]").unwrap();
        let tit_span_sel = Selector::parse("span.tit").unwrap();
        let info_sel = Selector::parse("span.bbs_info strong span").unwrap();

        let mut results = vec![];

        let onclick_re = Regex::new(r"post_to_url\('([^']+)'").unwrap();
        let id_re = Regex::new(r"(?:PMS_TSK_PBNC_ID|PMS_DMSY_PBNC_ID)=([^&]+)").unwrap();

        for li in doc.select(&li_sel) {
            // 제목 링크
            let a = match li.select(&tit_a_sel).next() {
                Some(a) => a,
                None => continue,
            };

            // 제목 텍스트
            let title = match a.select(&tit_span_sel).next() {
                Some(t) => t.text().collect::<String>().trim().to_string(),
                None => a.text().collect::<String>().trim().to_string(),
            };

            if title.is_empty() {
                continue;
            }

            let onclick = a.value().attr("onclick").unwrap_or("");
            let path = match onclick_re.captures(onclick) {
                Some(c) => c.get(1).unwrap().as_str().to_string(),
                None => continue,
            };

            let full_url = format!("https://ezone.iitp.kr{}", path);

            let ann_id = match id_re.captures(&full_url) {
                Some(c) => c.get(1).unwrap().as_str().to_string(),
                None => {
                    use std::hash::{Hash, Hasher};
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    full_url.hash(&mut hasher);
                    format!("{:x}", hasher.finish())
                }
            };

            // 접수기간
            let mut date = None;
            let mut deadline = None;

            if let Some(span) = li.select(&info_sel).next() {
                let period = span.text().collect::<String>().trim().to_string();
                let parts: Vec<&str> = period.split('~').collect();
                if parts.len() == 2 {
                    date = Some(parts[0].trim().to_string());
                    deadline = Some(parts[1].trim().to_string());
                }
            }

            let mut ann = Announcement::new(
                format!("iitp_{}", ann_id),
                title,
                full_url,
                self.source_name().to_string(),
            );
            ann.date = date;
            ann.deadline = deadline;

            results.push(ann);
        }

        Ok(results)
    }
}