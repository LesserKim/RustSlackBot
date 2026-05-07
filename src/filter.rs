use chrono::{Local, NativeDate};
use regex::Regex;
use crate::models::Announcement;

pub fn parse_deadline(s: &str) -> Option<NativeDate>
{
    if s.is_empty()
    {
        return None;
    }


    let specials = ["상시", "별도", "미정", "추후"];
    if specials.iter().any(|sp| s.contains(sp))
    {
        return None;
    }

    //숫자만 추출하는 필터
    let re = Regex::new(r"\d+").unwrap();
    let digits: String = re.find_iter(s).map(|m| m.as_str()).collect();

    if digits.len() >= 8
    {
        //yyyymmdd
        NativeDate::parse_from_str(&digits[..8], "%Y%m%d").ok()
    }

    else if digits.len() >= 6
    {
        //yymmdd
        NativeDate::parse_from_str(&digits[..6], "%y%m%d").ok()
    }
    else
    {
        None
    }
    


}