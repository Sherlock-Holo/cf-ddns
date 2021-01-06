use anyhow::Result;

use cf_ddns::run;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    run().await
}
