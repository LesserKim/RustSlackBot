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
        let ul_set = Selector::parse("div.board_list_area ul.list").unwrap();
        let li_sel = Selector::parse("li").unwrap();
        let a_sel = Selector::parse("div.tit_area a.tit").unwrap();
        let info_sel = Selector::parse("div.info_list_area li").unwrap();

        let mut results = vec![];

    }

}