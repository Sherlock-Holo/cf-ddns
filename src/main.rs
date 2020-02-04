use anyhow::Result;

use cf_ddns::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}

