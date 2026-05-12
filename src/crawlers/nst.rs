use scraper::{Html, Selector};
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct NstCrawler {
    timeout: u64,
}

impl NstCrawler {
    pub fn new(timeout: u64) -> Self {
        Self { timeout }
    }
}

impl Crawler for NstCrawler {
    fn source_name(&self) -> &str {
        "NST"
    }

    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>> {
        let url = "https://www.nst.re.kr/www/selectBbsNttList.do?bbsNo=18&key=60";
        let client = build_client(self.timeout);
        let html = client.get(url)
            .header("Referer", "https://www.nst.re.kr")
            .send()?
            .text()?;

        let doc = Html::parse_document(&html);
        let tbody_sel = Selector::parse("table.p-table tbody").unwrap();
        let tr_sel = Selector::parse("tr").unwrap();
        let td_sel = Selector::parse("td").unwrap();
        let subj_sel = Selector::parse("td.p-subject").unwrap();
        let a_sel = Selector::parse("a").unwrap();

        let mut results = vec![];

        let tbody = match doc.select(&tbody_sel).next() {
            Some(t) => t,
            None => return Ok(results),
        };

        for tr in tbody.select(&tr_sel) {
            let td_subj = match tr.select(&subj_sel).next() {
                Some(t) => t,
                None => continue,
            };

            let a = match td_subj.select(&a_sel).next() {
                Some(a) => a,
                None => continue,
            };

            let title = a.text().collect::<String>().trim().to_string();
            let href = a.value().attr("href").unwrap_or("");

            let full_url = if href.starts_with("./") {
                format!("https://www.nst.re.kr/www/{}", &href[2..])
            } else if href.starts_with("/") {
                format!("https://www.nst.re.kr{}", href)
            } else {
                href.to_string()
            };

            let ann_id = if full_url.contains("nttNo=") {
                full_url.split("nttNo=").nth(1).unwrap().split("&").next().unwrap().to_string()
            } else {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                full_url.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            };

            // 나머지 td에서 날짜/기관명 추출
            let mut date = None;
            let mut deadline = None;
            let mut agency = None;

            for td in tr.select(&td_sel) {
                let text = td.text().collect::<String>().trim().to_string();
                if text.len() == 10 && text.matches('-').count() == 2 {
                    if date.is_none() {
                        date = Some(text);
                    } else {
                        deadline = Some(text);
                    }
                } else if text.contains("연구원") || text.contains("연구회") {
                    agency = Some(text);
                }
            }

            let mut ann = Announcement::new(
                format!("nst_{}", ann_id),
                title,
                full_url,
                self.source_name().to_string(),
            );
            ann.date = date;
            ann.deadline = deadline;
            ann.agency = agency;

            results.push(ann);
        }

        Ok(results)
    }
}