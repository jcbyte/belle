use anyhow::Context;
use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
struct AFPRepo {
    name: String,
}

async fn get_latest_afp_repo_name() -> anyhow::Result<String> {
    let isa_afp_repo_list_endpoint = "https://foss.heptapod.net/api/v4/groups/isa-afp/projects?order_by=last_activity_at&sort=desc";

    let client = reqwest::Client::new();
    let projects: Vec<AFPRepo> = client
        .get(isa_afp_repo_list_endpoint)
        .header("User-Agent", "belle-client")
        .send()
        .await
        .context("Failed to send request to Hetapod")?
        .json()
        .await
        .context("Failed to parse JSON response from Hetapod")?;

    let re =
        Regex::new(r"^afp-[\d-]+$").context("Invalid regex pattern for AFP repository name")?;
    let latest_afp = projects
        .into_iter()
        .find(|p| re.is_match(&p.name))
        .context("No latest AFP repository could be found")?;

    return Ok(latest_afp.name);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let name = get_latest_afp_repo_name().await?;
    print!("name: {}", name);

    return Ok(());
}

// curl -L -o releases.toml https://foss.heptapod.net/isa-afp/afp-2025-2/-/raw/branch/default/metadata/releases.toml
