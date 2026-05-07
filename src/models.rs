use std::collections::HashMap;
use serde::{Deserialize, Serialize};

//매크로 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    pub id: String,
    pub title: String,
    pub url: String, 
    pub source: String,
    pub date: Option<String>,
    pub agency: Option<String>,
    pub deadline: Option<String>,
    pub matched_keywords: Vec<String>,
    pub extra : HashMap<String, String>,
}

impl Announcement {
    pub fn new(id: String, title: String, url: String, source: String) -> Self{
        Self{
            id, 
            title,
            url,
            source,
            date: None,
            agency: None,
            deadline: None,
            matched_keywords: vec![],
            extra: HashMap::new(),
        }
    }
}