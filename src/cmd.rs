use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cf-ddns", about = "An Cloudflare DDNS client.")]
pub struct Opt {
    #[structopt(short, long, help = "config path, - means read config from stdin")]
    pub config: PathBuf,

    #[structopt(long, help = "enable debug log")]
    pub debug: bool,
}
