use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::dns::DnsType;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub email: String,

    pub auth_key: String,

    pub domain: String,

    pub dns_type: DnsType,

    pub name: String,

    pub ttl: Option<u32>,

    pub proxied: Option<bool>,
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let mut reader: Box<dyn Read> = if path.as_ref() == OsStr::new("-") {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(path).context("read config failed")?)
    };

    let cfg = serde_yaml::from_reader::<_, Config>(&mut reader).context("parse config failed")?;

    Ok(cfg)
}
