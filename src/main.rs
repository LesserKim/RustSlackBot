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

    let crawlers = build_crawlers(cfg.request_timeout);

    for crawler in crawlers
    {
        let raw = crawler.safe_fetch();
        let matched = filter::filter_announcements(raw.clone(), &cfg.keywords);

        let new_ones: Vec<_> = matched.into_iter()
            .filter(|a| !seen.contains(&a.id))
            .collect();

        log::info(
            "[{}] 수집 {}건 -> 매칭 -> 신규 {}건",
            crawler.source_name(),
            raw.len(),
            new_ones.len()
        );

        all_new.extend(new_ones);
        thread::sleep(Duration::from_secs_f64(cfg.request_delay));
    }

    if !all_new.is_empty()
    {
        let notifier = slack::SlackNotifier::new(
            cfg.slack_bot_token.clone(),
            cfg.slack_channel.clone(),
        );
        let refs: Vec<_> = all_new.iter().collect();
        if notifier.send(&refs)
        {
            for ann in &all_new
            {
                seen.insert(ann.id.clone());
            }
            seen::save_seen(&cfg.seen_db_path, &seen);
        }
    } else {
        log::info!("새로운 매칭 공고 없음");
    }
    log::info!("크롤링 완료\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cfg = Config::load();

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--now")
    {
        log::info!("즉시 실행 모드");
        run_job(&cfg);
        return Ok(());
    }

    let scheduler = JobScheduler::new().await?;
    for time_str in &cfg.schedule_times {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            continue;
        }
        let hour: u32 = parts[0].parse().unwrap_or(9);
        let minute: u32 = parts[1].parse().unwrap_or(0);

        // cron 표현식: "초 분 시 일 월 요일"
        // Asia/Seoul = UTC+9, 서버 시간이 UTC면 9시간 빼야 함
        let cron = format!("0 {} {} * * *", minute, hour);

        let cfg_clone = cfg.clone();
        scheduler.add(Job::new(cron.as_str(), move |_, _| {
            run_job(&cfg_clone);
        })?).await?;

        log::info!("스케줄 등록: 매일 {} (UTC)", time_str);
    }

    scheduler.start().await?;
    log::info!("스케줄러 시작");

    // 무한 대기
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
