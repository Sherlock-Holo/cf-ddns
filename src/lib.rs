use anyhow::Result;
use log::LevelFilter;
use structopt::StructOpt;

use crate::cmd::Opt;
use crate::dns::{Client, DnsType};

mod cmd;
mod config;
mod dns;
mod ip;

pub async fn run() -> Result<()> {
    let opt = Opt::from_args();

    log_init(opt.debug);

    let cfg = config::load_config(&opt.config)?;

    log::info!("email: {}", cfg.email);
    log::info!("domain: {}", cfg.domain);
    log::info!("name: {}", cfg.name);
    log::info!("type: {:?}", cfg.dns_type);

    let content = match cfg.dns_type {
        DnsType::A => ip::get_local_ipv4_ip().await?.to_string(),
        DnsType::AAAA => ip::get_local_ipv6_ip().await?.to_string(),
    };

    log::info!("IP: {}", content);

    let client = Client::new(cfg);

    client.run(&content).await
}

fn log_init(debug: bool) {
    let mut builder = pretty_env_logger::formatted_timed_builder();

    if debug {
        builder.filter_level(LevelFilter::Debug);
    } else {
        builder.filter_level(LevelFilter::Info);
    }

    builder.init();
}
