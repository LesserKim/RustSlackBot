use std::collections::HashMap;
use scraper::{Html, Selector};
use regex::Regex;
use super::crawlers::base::build_client;


pub fn fetch_detail(url:&str, content_selector:&str, timeout: u64) -> HashMap<String, String>
{
    let mut result = HashMap::new();

    let client = build_client(timeout);
    let html = match client.get(url).send().and_then(|r| r.text)
    {
        Ok(h) => h,
        Err(_) => return result,
    };

    let doc = Html::parse_document(&html);
    let sel = match Selector::parse(content_selector)
    {
        Ok(s) => s,
        Err(_) => return result,
    };

    let content = match doc.select(&sel).next()
    {
        Some(c) => c,
        None => return result,
    };

    //테이블에서 마감일 추출
    
}