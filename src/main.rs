mod models;
mod config;
mod seen;
mod filter;
mod slack;
mod extractor;
mod crawlers;
mod summarizer;

use std::thread;
use std::time::Duration;
use chrono::Local;

use crate::config::Config;
use crate::crawlers::base::Crawler;
use crate::crawlers::{
    ntis::NtisCrawler,
    kisa::KisaCrawler,
    iitp::IitpCrawler,
    kisti::KistiCrawler,
    nst::NstCrawler,
    krit::KritCrawler,
    iris::IrisCrawler,
    etri::EtriCrawler,
    msit::MsitCrawler,
};

fn build_crawlers(timeout: u64) -> Vec<Box<dyn Crawler>> {
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

fn run_job(cfg: &Config) {
    let now = Local::now().format("%Y-%m-%d %H:%M");
    log::info!("{}", "=".repeat(50));
    log::info!("크롤링 시작: {}", now);
    log::info!("{}", "=".repeat(50));

    let mut seen = seen::load_seen(&cfg.seen_db_path);
    let mut all_new = vec![];

    let crawlers = build_crawlers(cfg.request_timeout);

    for crawler in &crawlers {
        let raw = crawler.safe_fetch();
        let raw_len = raw.len();
        let matched = filter::filter_announcements(raw, &cfg.keywords);

        let new_ones: Vec<_> = matched.into_iter()
            .filter(|a| !seen.contains(&a.id))
            .collect();

        log::info!(
            "[{}] 수집 {}건 -> 매칭 -> 신규 {}건",
            crawler.source_name(),
            raw_len,
            new_ones.len()
        );

        all_new.extend(new_ones);
        thread::sleep(Duration::from_secs_f64(cfg.request_delay));
    }

    if !all_new.is_empty() {
        let notifier = slack::SlackNotifier::new(
            cfg.slack_bot_token.clone(),
            cfg.slack_channel.clone(),
        );
        let refs: Vec<_> = all_new.iter().collect();
        if notifier.send(&refs) {
            for ann in &all_new {
                seen.insert(ann.id.clone());
            }
            seen::save_seen(&cfg.seen_db_path, &seen);
        }
    } else {
        log::info!("새로운 매칭 공고 없음");
    }

    log::info!("크롤링 완료\n");
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cfg = Config::load();

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--now") {
        log::info!("즉시 실행 모드");
        run_job(&cfg);
        return;
    }

    // 스케줄러 모드: 단순 루프 방식
    log::info!("스케줄러 시작");
    for time_str in &cfg.schedule_times {
        log::info!("스케줄 등록: 매일 {}", time_str);
    }

    loop {
        let now = Local::now().format("%H:%M").to_string();
        if cfg.schedule_times.contains(&now) {
            run_job(&cfg);
            // 같은 분에 다시 실행 안 되게 61초 대기
            thread::sleep(Duration::from_secs(61));
        } else {
            thread::sleep(Duration::from_secs(30));
        }
    }
}