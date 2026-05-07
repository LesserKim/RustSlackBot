use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn load_seen(path: &str) -> HashSet<String>
{
    if !Path::new(path).exists(){
        return HashSet::new();
    }
    match fs::read_to_string(path)
    {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashSet::new(),
    }
}

pub fn save_seen(path: &str, seen: &HashSet<String>)
{
    if let Ok(json) = serde_json::to_string_pretty(seen){
        let _ = fs::write(path, json);
    }
}

pub fn filter_new<'a>(
    announcements: &'a [crate::models::Announcement],
    seen: &HashSet<String>,
) -> Vec<&'a crate::models::Announcement>{
    announcements.iter().filter(|a| !seen.contains(&a.id)).collect()
}

