use anyhow::{Context, anyhow};
use regex::Regex;

#[derive(Debug)]
pub struct RootFileDependencies {
    pub parent: String,
    pub sessions: Vec<String>,
}

impl RootFileDependencies {
    // Iterate over parent + external sessions
    pub fn iter_all(&self) -> impl Iterator<Item = &String> {
        return std::iter::once(&self.parent).chain(&self.sessions);
    }
}

/// Extract the parent and any external sessions from an Isabelle/HOL ROOT file using static analysis
pub fn extract_root_deps(content: &str) -> anyhow::Result<RootFileDependencies> {
    // Match and capture the parent session (after '=') if it is quoted or unquoted
    let parent_re = Regex::new(
        r#"(?x)
        session\s+.*?\s*=\s* # Skip past the 'session [name] =' 
        (?:"(?P<q>[^"]+)"|(?P<u>[\w\-]+)) # Capture quoted or unquoted parent
    "#,
    )
    .context("Failed to create parent Regex pattern")?;

    let parent_captures = parent_re
        .captures(content)
        .ok_or_else(|| anyhow!("Missing 'session' definition."))?;
    let parent = parent_captures
        .name("q")
        .or(parent_captures.name("u"))
        .map(|s| s.as_str().to_string())
        .ok_or_else(|| anyhow!("Failed to capture parent"))?;

    // Find the block starting with 'sessions' until the next block keyword or end of string.
    let sessions_block_re =
        Regex::new(r#"(?s)\bsessions\b\s*(.*?)(?:\btheories\b|\bdocument_files\b|\bdirectories\b|\boptions\b|$)"#)
            .context("Failed to create session block Regex patten")?;

    let mut sessions = Vec::new();
    if let Some(block) = sessions_block_re.captures(content) {
        // Extract quoted names within the sessions block, the external sessions
        let quote_re = Regex::new(r#""([^"]+)""#).context("Failed to create session regex pattern")?;
        for capture in quote_re.captures_iter(&block[1]) {
            sessions.push(capture[1].to_string());
        }
    }

    return Ok(RootFileDependencies { parent, sessions });
}
