use scraper::{Html, Selector};
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct KisaCrawler {
    timeout: u64,
}

impl KisaCrawler {
    pub fn new(timeout: u64) -> Self {
        Self { timeout }
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

            let mut ann = Announcement::new(
                format!("kisa_{}", ann_id),
                title,
                full_url,
                self.source_name().to_string(),
            );
            ann.date = date;

            results.push(ann);
        }

        Ok(results)
    }
}



