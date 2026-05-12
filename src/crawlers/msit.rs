use scraper::{Html, Selector};
use regex::Regex;
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct MsitCrawler {
    timeout: u64,
}

impl MsitCrawler {
    pub fn new(timeout: u64) -> Self {
        Self { timeout }
    }
}

impl Crawler for MsitCrawler {
    fn source_name(&self) -> &str {
        "MSIT"
    }

    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>> {
        let url = "https://www.msit.go.kr/bbs/list.do?sCode=user&mId=311&mPid=121&pageIndex=1&bbsSeqNo=100&searchOpt=ALL&searchTxt=보안";
        let client = build_client(self.timeout);
        let html = client.get(url)
            .header("Referer", "https://www.msit.go.kr")
            .send()?
            .text()?;

        let doc = Html::parse_document(&html);
        let a_sel = Selector::parse("div.board_list div.toggle > a").unwrap();
        let title_sel = Selector::parse("p.title").unwrap();
        let date_sel = Selector::parse("div.date").unwrap();
        let meta_sel = Selector::parse("div.meta").unwrap();

        let mut results = vec![];

        let onclick_re = Regex::new(r"fn_detail\((\d+)\)").unwrap();

        for a in doc.select(&a_sel) {
            let onclick = a.value().attr("onclick").unwrap_or("");
            let ntt_seq_no = match onclick_re.captures(onclick) {
                Some(c) => c.get(1).unwrap().as_str().to_string(),
                None => continue,
            };

            let url = format!(
                "https://www.msit.go.kr/bbs/view.do?sCode=user&mId=311&mPid=121&bbsSeqNo=100&nttSeqNo={}",
                ntt_seq_no
            );

            let title = match a.select(&title_sel).next() {
                Some(t) => t.text().collect::<String>().trim().to_string(),
                None => continue,
            };

            let date = a.select(&date_sel).next()
                .map(|t| t.text().collect::<String>().trim().to_string());

            let agency = a.select(&meta_sel).next()
                .map(|t| t.text().collect::<String>().trim().to_string());

            let mut ann = Announcement::new(
                format!("msit_{}", ntt_seq_no),
                title,
                url,
                self.source_name().to_string(),
            );
            ann.date = date;
            ann.agency = agency;

            results.push(ann);
        }

        Ok(results)
    }
}