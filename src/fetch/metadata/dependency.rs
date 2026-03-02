use std::iter;

use anyhow::Context;

#[derive(Debug, Clone)]
pub struct RootFileSession {
    pub name: String,
    pub parent: String,
    pub sessions: Vec<String>,
}

impl RootFileSession {
    pub fn iter_all(&self) -> impl Iterator<Item = &String> {
        return iter::once(&self.parent).chain(self.sessions.iter());
    }
}

/// Strip nested Isabelle comments "(* ... *)" and formal "\<comment> \<open> ... \<close>" comments
fn strip_comments(input: &str) -> String {
    let mut result = String::new();
    let mut depth = 0;

    let mut i = 0;
    while i < input.len() {
        let rest = &input[i..];

        // Check for opening comment
        if rest.starts_with("(*") {
            depth += 1;
            i += 2;
            continue;
        } else if rest.starts_with("\\<open>") {
            depth += 1;
            i += 7;
            continue;
        }

        // Check for closing comment
        if rest.starts_with("*)") {
            depth -= 1;
            i += 2;
            continue;
        } else if rest.starts_with("\\<close>") {
            depth -= 1;
            i += 8;
            continue;
        }

        // Ignore \<comment> tags
        if rest.starts_with("\\<comment>") {
            i += 10;
            continue;
        }

        if let Some(c) = rest.chars().next() {
            // Record the character if not currently inside a comment
            if depth == 0 {
                result.push(c);
            }

            // Skip the next character safely
            i += c.len_utf8();
        }
    }
    return result;
}

/// Parse an identifier: either quoted string or unquoted alphanumeric
fn parse_identifier(input: &str) -> Option<(&str, &str)> {
    let input = input.trim_start();

    if input.starts_with('"') {
        // Parse quoted identifier
        if let Some(end_quote) = input[1..].find('"') {
            let id = &input[1..end_quote + 1];
            let rest = &input[end_quote + 2..];
            return Some((id, rest));
        }
    } else {
        // Parse unquoted identifier (alphanumeric+-._)
        let mut end = 0;
        for ch in input.chars() {
            match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' | '+' => {
                    end += ch.len_utf8();
                }
                _ => break,
            }
        }
        if end > 0 {
            return Some((&input[..end], &input[end..]));
        }
    }

    return None;
}

pub fn parse_root(root: &str) -> anyhow::Result<Vec<RootFileSession>> {
    let clean_root = strip_comments(root);
    let mut sessions: Vec<RootFileSession> = Vec::new();

    // Skip the first block as this will be preamble
    let session_blocks = clean_root.split("\nsession ").skip(1);
    for session_block in session_blocks {
        // The name is th first thing after the session
        let (name, rest) = parse_identifier(session_block).context("The session name could not be parsed")?;

        // This skips any notes after the name
        let (_, rest) = rest
            .split_once("=")
            .context("The session header could not be skipped during parsing")?;

        // The parent session is given after the "="
        let (parent, rest) = parse_identifier(rest).context("The session parent could not be parsed")?;

        let mut dependencies: Vec<String> = Vec::new();
        // Skip any details and go to where sessions are defined (if any)
        if let Some((_, session_rest)) = rest.split_once("sessions") {
            let mut rest = session_rest;

            while let Some((dep, next_rest)) = parse_identifier(rest) {
                if matches!(dep, "theories" | "document_files" | "directories" | "options") {
                    break;
                }

                dependencies.push(dep.to_string());
                rest = next_rest;
            }
        };

        sessions.push(RootFileSession {
            name: name.to_string(),
            parent: parent.to_string(),
            sessions: dependencies,
        });
    }

    return Ok(sessions);
}
