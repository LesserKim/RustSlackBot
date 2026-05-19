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
        let li_sel = Selector::parse("ul.basic_bbs li.clearfix").unwrap();
        let tit_sel = Selector::parse("span.tit").unwrap();
        let a_sel = Selector::parse("a[onclick]").unwrap();
        let info_span_sel = Selector::parse("span.bbs_info span").unwrap();

        let mut results = vec![];

        let onclick_re = Regex::new(r"post_to_url\('([^']+)'").unwrap();

        for li in doc.select(&li_sel) {
            let title = match li.select(&tit_sel).next() {
                Some(t) => t.text().collect::<String>().trim().to_string(),
                None => continue,
            };

            if title.is_empty() {
                continue;
            }

            let a = match li.select(&a_sel).next() {
                Some(a) => a,
                None => continue,
            };

            let onclick = a.value().attr("onclick").unwrap_or("");
            let path = match onclick_re.captures(onclick) {
                Some(c) => c.get(1).unwrap().as_str().to_string(),
                None => continue,
            };

            let full_url = format!("https://ezone.iitp.kr{}", path);

            let ann_id_re = Regex::new(r"(?:PMS_TSK_PBNC_ID|PMS_DMSY_PBNC_ID|PMS_TK_PBNC_ID)=([^&]+)").unwrap();
            let ann_id = match ann_id_re.captures(&full_url) {
                Some(c) => c.get(1).unwrap().as_str().to_string(),
                None => {
                    use std::hash::{Hash, Hasher};
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    full_url.hash(&mut hasher);
                    format!("{:x}", hasher.finish())
                }
            };

            let mut date = None;
            let mut deadline = None;

            if let Some(info_span) = li.select(&info_span_sel).next() {
                let period = info_span.text().collect::<String>().trim().to_string();
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