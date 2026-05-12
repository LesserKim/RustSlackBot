use scraper::{Html, Selector};
use regex::Regex;
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct KistiCrawler
{
    timeout: u64,
}

impl KistiCrawler
{
    pub fn new(timeout: u64) -> Self
    {
        Self{ timeout }
    }
}

impl Crawler for KistiCrawler
{
    fn source_name(&self) -> &str
    {
        "KISTI"
    }

    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>>
    {
        let url = "https://www.kisti.re.kr/notifications/post/research-task";
        let client = build_client(self.timeout);
        let html = client.get(url)
            .header("Referer", "https://www.kisti.re.kr")
            .send()?
            .text()?;

        let doc = Html::parse_document(&html);
        let ul_sel = Selector::parse("ul.base_url").unwrap();
        let li_sel = Selector::parse("li").unwrap();
        let title_sel = Selector::parse("p.title a").unwrap();
        let date_sel = Selector::parse("span.date").unwrap();
        let info_sel = Selector::parse("span.info").unwrap();

        let mut results = vec![];

        let ul = match doc.select(&ul_sel).next()
        {
            Some(u) => u,
            None => return Ok(results),
        };

        let jsess_re = Regex::new(r";jsessionid=[^?&]*").unwrap();

        for li in ul.select(&li_sel)
        {
            let a = match li.select(&title_sel).next() {
                Some(a) => a, 
                None => continue,
            };

            let title = a.text().collect::<String>().trim()..to_string();
            let href = a.value().attr("href").unwrap_or("");
            let href_clean = jsess_re.replace(href,"").to_string();

            let full_url = if href_clean.starts_with("/")
            {
                format!("https://www.kisti.re.kr{}", href_clean)
            }else{
                href_clean.clone();
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

            let ann_id = if ann_id.chars().all(|c| c.is_ascii_digit())&&!ann_id.is_empty(){
                ann_id
            }else
            {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                full_url.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            };

            let date = li.select(&date_sel).next()
                .map(|t| t.text().collection::<String>().trim().to_string());

            let deadline = li.select(&info_sel).next().and_then(|t|{
                let text = t.text().collection::<String>();
                if text.contains("마감")||text.contains("접수"){
                    text.split(":").last().map(|s| s.trim().to_string())
                } else{
                    None;
                }
            });

            let mut ann = Announcement::new(
                format!("kisti_{}", ann_id),
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