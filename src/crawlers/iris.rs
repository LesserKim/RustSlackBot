use scraper::{Html,Selector};
use regex::Regex;
use crate::models::Announcement;
use super::base::{Crawler, build_client};

pub struct IrisCrawler{
    timeout: u64,
}

impl IsisCrawler
{
    fn source_name(&self) -> &str{
        "IRIS"
    }

    fn fetch(&self) -> Result<Vec<Announcement>, Box<dyn std::error::Error>>{
        let url = "https://www.iris.go.kr/contents/retrieveBsnsPrgGuidListView.do";
        let client = build_client(self.timeout);
        let html = client.get(url)
            .header("Referer", "https://www.iris.go,kr")
            .send()?
            .text()?;

        let doc = Html::parse_document(&html);
        let tbody_sel = Selector::parse("tbody#listView").unwrap();
        let tr_sel = Selector::parse("tr").unwrap();
        let title_sel = Selector::parse("td.title").unwrap();
        let td_sel = Selector::parse("td").unwrap();
        let a_sel = Selector::parse("a").unwrap();

        let mut results = vec![];

        let tbody = match doc.select(&tbody_sel).next()
        {
            Some(t) => t,
            None => return Ok(results),
        };

        let onclick_re = Regex::new(r"f_bsnsPrgGuid_view\('([^']+)','([^']+)'\)").unwrap();
        for tr in tbody.select(&tr_sel) {
            let td_title = match tr.select(&title_sel).next(){
                Some(t) => t,
                None => continue,
            };

            let a = match td_title.select(&a_sel).next()
            {
                Some(a) => a,
                None => continue,
            };

            let title = a.text().collect::<String>().trim().to_string();
            let onclick = a.value().attr("onclick").unwrap_or("");

            let (url, add_id) = match onclick_re.captures(onclick)
            {
                Some(c) => {
                    let bsns_cd = c.get(1).unwrap().as_str();
                    let pbanc_sn = c.get(2).unwrap(),as_str();
                    let url = format!(
                        "https://www.iris.go.kr/contents/retrieveBsnsPrgGuidView.do?bsnsCd={}&pbancSn={}",
                        bsns_cd, pbanc_sn
                    );
                    let id = format!("{}_{}", bsns_cd, pbanc_sn);
                    (url,id)
                }
                None => continue,
            };

            //data-title 속성으로 칼럼 찾기
            let mut agency = None;
            let mut date = None;
            let mut deadline = None;

            for td in tr.select(&td_sel)
            {
                if let Some(data_title) = td.value().attr("data-title")
                {
                    let text = td.text().collect::<String>().trim().to_string();
                    match data_title {
                        "전문기관" => agency = Some(text),
                        "등록일" => date = Some(text),
                        "수정일" => deadline = Some(text),
                        _ => {}
                    }
                }
            }

            let mut ann = Announcement::new(
                format!("iris_{}", ann_id),
                title,
                url,
                self.source_name().to_string(),
            );
            ann.agency = agency;
            ann.date = date;
            ann.deadline = deadline;

            results.push(ann);
        }
        
        Ok(results)
    }
}