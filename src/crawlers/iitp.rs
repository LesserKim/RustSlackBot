use scraper::{Html, Selector};
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct IitpCrawler
{
    timeout: u64,
}

impl IitpCrawler
{
    pub fn new(timeout: u64)->Self
    {
        Self{timeout}
    }
}

impl Crawler for IitpCrawler
{
    fn source_name(&self) -> &str
    {
        "IITP"
    }

    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>>
    {
        let url = "https://www.iitp.kr/web/lay1/program/S1T44C51/iris/list.do";
        let client = build_client(self.timeout);
        let html = client.get(url)
            .header("Referer", "https://www.iitp.kr")
            .send()?
            .text()?;

        let doc = Html::parse_document(&html);
        let ul_sel = Selector::parse("div.board_list_area ul.list").unwrap();
        let li_sel = Selector::parse("li").unwrap();
        let a_sel = Selector::parse("div.tit_area a.tit").unwrap();
        let info_sel = Selector::parse("div.info_list_area li").unwrap();

        let mut results = vec![];

        let ul = match doc.select(&ul_sel).next()
        {
            Some(u) => u,
            None => return Ok(results),
        };

        for li in ul.select(&li_sel)
        {
            let a = match li.select(&a_sel).next()
            {
                Some(a) => a,
                None => continue,
            };

            let title = a.text().collect::<String>().trim().to_string();
            let href = a.value().attr("href").unwrap_or("");

            let full_url = if href.starts_with("./")
            {
                format!("https://www.iitp.kr/web/lay1/program/S1T44C51/iris/{}", &href[2..])
            } else if href.starts_with("/")
            {
                format!("https://www.iitp.kr{}", href)
            }
            else
            {
                href.to_string()
            };

            let ann_id = if full_url.contains("id=")
            {
                full_url.split("id=").nth(1).unwrap().split("&").next().unwrap().to_string()
            } else
            {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                full_url.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            };

            let mut date = None;
            let mut deadline = None;
            let mut agency = None;

            for info_li in li.select(&info_sel)
            {
                let text = info_li.text().collect::<String>();
                let text = text.trim();
                if text.contains("접수") || text.contains("신청")
                {
                    deadline = text.split(":").last().map(|s| s.trim().to_string()); 
                }
                else if text.contains("공고일") || text.contains("등록")
                {
                    date = text.split(":").last().map(|s| s.trim().to_string());
                }
                else if text.contains("기관") || text.contains("주관")
                {
                    agency = text.split(":").last().map(|s| s.trim().to_string());
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
            ann.agency = agency;

            results.push(ann);
        }
        Ok(results)
    }
}