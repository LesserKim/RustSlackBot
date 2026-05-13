mod models;
mod config;
mod seen;
mod filter;
mod slack;
mod extractor;
mod crawlers;

use std::collections::HashSet;
use std::thread;
use std::time::Duration;
use chrono::Local;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::config::Config;
use crate::crawlers::base::Crawler;
use crate::crawlers::krit::KritCrawler;
use crate::crawlers::{
    ntis::NtisCrawler,
    kisa::KisaCrawler,
    iitp::IitpCrawler,
    kisti::KistiCrawler,
    nst::NstCrawler,
    iris::IrisCrawler,
    etri::EtriCrawler,
    msit::MsitCrawler,
};

fn build_crawlers(timeout: u64) -> Vec<Box<dyn Crawler + Send + Sync>>
{
    vec![
        Box::new(NtisCrawler::new(timeout)),
        Box::new(KisaCrawler::new(timeout)),
        Box::new(IitpCrawler::new(timeout)),
        Box::new(KistiCrawler::new(timeout)),
        Box::new(NstCrawler::new(timeout)),
        Box::new(KritCrawler::new(timeout)),
        Box::new(IrisCrawler::new(timeout)),
        Box::new(EtriCrawler::new(timeout)),
        Box::new(MsitCrawler::new(timeout)),
    ]
}

fn run_job(cfg: &Config)
{
    let now = Local::now().format("%Y-%m-%d %H:%M");
    log::info!("{}", "=".repeat(50));
    log::info!("크롤링 시작: {}", now);
    log::info!("{}", "=".repeat(50));

    let mut seen = seen::load_seen(&cfg.seen_db_path);
    let mut all_new = vec![];

    
}

fn main() {
}
