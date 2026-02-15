mod fetch;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fetch::update().await?;

    return Ok(());
}
