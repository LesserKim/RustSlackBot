use scraper::{Html, Selector};
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct NtisCrawler
{
    timeout: u64,
}

impl NtisCrawler
{
    pub fn new(timeout: u64) -> Self
    {
        Self{timeout}
    }
}

impl Crawler for NtisCrawler
{
    fn source_name(&self) -> &str{
        "NTIS"
    }

    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>>
    {
        let url = "https://www.ntis.go.kr/rndgate/eg/un/ra/mng.do";
        let client = build_client(self.timeout);
        let html = client.get(url)
            .header("Referer", "https://www.ntis.go.kr")
            .send()?
            .text()?;

        let doc = Html::parse_document(&html);
        let tbody_sel = Selector::parse("table.basic_list tbody").unwrap();
        let tr_sel = Selector::parse("tr").unwrap(); //Selector: CSS 선택자, table 태그 중 클래스가 tbl_board인 것 안에 있는 tbody찾음
        let td_sel = Selector::parse("td").unwrap();
        let a_sel = Selector::parse("a").unwrap();

        let mut results = vec![];

        let tbody = match doc.select(&tbody_sel).next()
        {
            Some(t) => t, //테이블 있으면 계속 진행
            None => return Ok(results), //없으면 None 대신 빈리스트 반환
        };

        for tr in tbody.select(&tr_sel)
        {
            let mut title_td = None;
            let mut date_td = None;
            let mut agency_td = None;
            let mut deadline_td = None;

            for td in tr.select(&td_sel)
            {
                if let Some(data_title) = td.value().attr("data-title")
                {
                    match data_title.trim()
                    {
                        "공고명" => title_td = Some(td),
                        "등록일" => date_td = Some(td),
                        "기관명" => agency_td = Some(td),
                        "마감일" => deadline_td = Some(td),
                        _ => {},
                    }
                }
            }

            let title_td = match title_td
            {
                Some(a) => a, 
                None => continue,
            };

            let a = match title_td.select(&a_sel).next()
            {
                Some(a) => a,
                None => continue,
            };

            let title = a.text().collect::<String>().trim().to_string();
            let href = a.value().attr("href").unwrap_or("");
            let full_url = if href.starts_with("/"){ //상대경로를 절대경로로 변환시켜줌
                format!("https://www.ntis.go.kr{}", href)
            } else {
                href.to_string()
            };

            let ann_id = if full_url.contains("roRndUid=")
            {
                full_url.split("roRndUid=").nth(1).unwrap().split("&").next().unwrap().to_string()
            } else {
                format!("{:x}", md5_hash(&full_url))
            };

            let mut ann = Announcement::new( //새 공고 객체 생성
                format!("ntis_{}", ann_id), 
                title,
                full_url,
                self.source_name().to_string(),
            );

            let mut ann = Announcement::new(
                format!("ntis_{}", ann_id),
                title,
                full_url,
                self.source_name().to_string(),
            );
            ann.agency = agency_td.map(|t| t.text().collect::<String>().trim().to_string());
            ann.date = date_td.map(|t| t.text().collect::<String>().trim().to_string());
            ann.deadline = deadline_td.map(|t| t.text().collect::<String>().trim().to_string());

            results.push(ann); //리스트에 추가
        }
        Ok(results)
    }
}

fn md5_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}