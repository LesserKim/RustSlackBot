use scraper::{Html, Selector};
use regex::Regex;
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct KritCrawler {
    timeout: u64,
}

impl KritCrawler {
    pub fn new(timeout: u64) -> Self {
        Self { timeout }
    }
}

impl Crawler for KritCrawler {
    fn source_name(&self) -> &str {
        "KRIT"
    }

    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>> {
        let url = "https://dtims.krit.re.kr/vps/OINF_RndProjList.do";
        let client = build_client(self.timeout);
        let html = client.get(url)
            .header("Referer", "https://dtims.krit.re.kr")
            .send()?
            .text()?;

        let doc = Html::parse_document(&html);
        let tbody_sel = Selector::parse("table[data-skin='board_list'] tbody").unwrap();
        let tr_sel = Selector::parse("tr").unwrap();
        let td_sel = Selector::parse("td").unwrap();

        let mut results = vec![];

        let tbody = match doc.select(&tbody_sel).next() {
            Some(t) => t,
            None => return Ok(results),
        };

        let onclick_re = Regex::new(r"fncBusinessInfo\('([^']+)'\)").unwrap();

        for tr in tbody.select(&tr_sel) {
            let onclick = tr.value().attr("onclick").unwrap_or("");
            let rnd_no = match onclick_re.captures(onclick) {
                Some(c) => c.get(1).unwrap().as_str().to_string(),
                None => continue,
            };

            let url = format!("https://dtims.krit.re.kr/vps/OINF_RndProjView.do?rndNo={}", rnd_no);

            // 제목: title 속성이 있는 td 우선, 없으면 텍스트가 가장 긴 td
            let tds: Vec<_> = tr.select(&td_sel).collect();
            let title = tds.iter()
                .find_map(|td| td.value().attr("title"))
                .map(|s| s.trim().to_string())
                .or_else(|| {
                    tds.iter()
                        .max_by_key(|td| td.text().collect::<String>().len())
                        .map(|td| td.text().collect::<String>().trim().to_string())
                });

            let title = match title {
                Some(t) if !t.is_empty() => t,
                _ => continue,
            };

            let agency = tds.last()
                .map(|t| t.text().collect::<String>().trim().to_string());

            let mut ann = Announcement::new(
                format!("krit_{}", rnd_no),
                title,
                url,
                self.source_name().to_string(),
            );
            ann.agency = agency;

            results.push(ann);
        }

        Ok(results)
    }
}