use scraper::{Html, Selector};
use regex::Regex;
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct EtriCrawler {
    timeout: u64,
}

impl EtriCrawler {
    pub fn new(timeout: u64) -> Self {
        Self { timeout }
    }
}

impl Crawler for EtriCrawler {
    fn source_name(&self) -> &str {
        "ETRI"
    }

    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>> {
        let url = "https://ebid.etri.re.kr/ebid/main.do?tabId=&g2b_conn=&dev=ebidList&biNo=&loginpage=Y&login_regnum=&pgmUrl=.%2Febid%2FebidCustProgressList.do&pageGb=";
        let client = build_client(self.timeout);
        let html = client.get(url)
            .header("Referer", "https://ebid.etri.re.kr")
            .send()?
            .text()?;

        let doc = Html::parse_document(&html);
        let tr_sel = Selector::parse("tr[onmouseover]").unwrap();
        let td_sel = Selector::parse("td").unwrap();

        let mut results = vec![];

        let onclick_re = Regex::new(r"fnDetailCustView\('([^']+)'\)").unwrap();
        let date_re = Regex::new(r"^\d{4}-\d{2}-\d{2}").unwrap();

        for tr in doc.select(&tr_sel) {
            let tds: Vec<_> = tr.select(&td_sel).collect();
            if tds.len() < 3 {
                continue;
            }

            // onclick에서 공고번호 추출
            let mut bi_no: Option<String> = None;
            for td in &tds {
                if let Some(onclick) = td.value().attr("onclick") {
                    if let Some(c) = onclick_re.captures(onclick) {
                        bi_no = Some(c.get(1).unwrap().as_str().to_string());
                        break;
                    }
                }
            }

            let bi_no = match bi_no {
                Some(b) => b,
                None => continue,
            };

            let url = format!("https://ebid.etri.re.kr/ebid/ebidCustProgressView.do?biNo={}", bi_no);

            // 제목: 텍스트가 가장 긴 td
            let title = tds.iter()
                .max_by_key(|td| td.text().collect::<String>().len())
                .map(|td| td.text().collect::<String>().trim().trim_matches('"').trim().to_string())
                .unwrap_or_default();

            if title.is_empty() {
                continue;
            }

            // 날짜 추출
            let mut dates = vec![];
            for td in &tds {
                let text = td.text().collect::<String>();
                let text = text.trim();
                if date_re.is_match(text) {
                    dates.push(text[..10].to_string());
                }
            }

            let date = dates.first().cloned();
            let deadline = dates.get(1).cloned();

            let mut ann = Announcement::new(
                format!("etri_{}", bi_no),
                title,
                url,
                self.source_name().to_string(),
            );
            ann.date = date;
            ann.deadline = deadline;

            results.push(ann);
        }

        Ok(results)
    }
}