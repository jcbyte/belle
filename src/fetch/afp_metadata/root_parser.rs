use std::{borrow::Cow, iter};

use crate::fetch::afp_metadata::error::{RootParserContext, RootParserError};

#[derive(Debug, Clone)]
pub struct RootFileSession {
    pub name: String,
    pub parent: String,
    pub sessions: Vec<String>,
}

impl RootFileSession {
    pub fn iter_all(&self) -> impl Iterator<Item = &String> {
        iter::once(&self.parent).chain(self.sessions.iter())
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
    result
}

/// Parse an identifier: either quoted string or unquoted alphanumeric
fn parse_identifier(input: &str) -> Option<(&str, &str)> {
    let input = input.trim_start();

    if let Some(stripped) = input.strip_prefix('"') {
        // Parse quoted identifier
        if let Some(end_quote) = stripped.find('"') {
            let id = &stripped[0..end_quote];
            let rest = &stripped[(end_quote + 1)..];
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

    None
}

pub fn parse_root(root: &str) -> Result<Vec<RootFileSession>, RootParserError> {
    let clean_root = strip_comments(root);
    let mut sessions: Vec<RootFileSession> = Vec::new();

    // Skip the first block as this will be preamble
    let session_blocks = clean_root.split("\nsession ").skip(1);
    for session_block in session_blocks {
        // The name is th first thing after the session
        let (name, rest) = parse_identifier(session_block).report_failed_parsing("session name")?;

        // This skips any notes after the name
        let (_, rest) = rest.split_once("=").report_failed_parsing("session header")?;

        // The parent session is given after the "="
        let (parent, rest) = parse_identifier(rest).report_failed_parsing("session parent")?;

        // Remove the description part of the session in case it contains "sessions"
        let session_body_rest = if let Some((before_desc, after_desc)) = rest.split_once("description") {
            let (_description, _desc) = parse_identifier(after_desc).report_failed_parsing("session description")?;
            // Rebuild the rest of ROOT file excluding the description block
            Cow::Owned(format!("{}{}", before_desc, after_desc))
        } else {
            // Use Cow to remove need to clone, when rebuilding in one branch
            Cow::Borrowed(rest)
        };

        let mut dependencies: Vec<String> = Vec::new();
        // Skip any details and go to where sessions are defined (if any)
        if let Some((_, session_rest)) = session_body_rest.split_once("sessions") {
            let mut rest = session_rest;

            // Once reaching the next listed block we have gone though all sessions
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

    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stripping_comments() {
        let text = r"
          Lorem ipsum dolor sit amet, consectetur adipiscing elit. (* This is an Isabelle/HOL Comment!! *) Praesent semper quis sapien sed imperdiet.
          Morbi lacinia diam nulla, nec semper ex dictum non. Morbi interdum ut metus id pulvinar. \<open> This is the secondary type of Isabelle/HOL Comment!! \<close>
          Curabitur tempus aliquam ullamcorper. Aliquam ullamcorper lacus felis, id faucibus dolor volutpat quis.
          Fusce et eros eu nulla gravida venenatis nec nec orci. (* This is an secondary Isabelle/HOL Comment!! *) Aliquam in felis ac libero suscipit pretium facilisis sit amet ante.
          Quisque ullamcorper libero ut mollis tincidunt. \<comment> \<open> This is the secondary type of Isabelle/HOL Comment!! (with a comment tag) \<close> Quisque commodo tincidunt urna vel molestie. 
        ";

        let stripped = strip_comments(text);

        // Make sure comments and comment locator characters are stripped
        assert!(!stripped.contains("Isabelle"));
        assert!(!stripped.contains("(*"));
        assert!(!stripped.contains("*)"));
        assert!(!stripped.contains(r"\<open>"));
        assert!(!stripped.contains(r"\<close>"));
        assert!(!stripped.contains(r"\<comment>"));

        // Verify the text remains
        assert!(stripped.contains("Lorem ipsum dolor sit amet, consectetur adipiscing elit."));
        assert!(stripped.contains("Aliquam ullamcorper lacus felis, id faucibus dolor volutpat quis."));
        assert!(stripped.contains("Quisque commodo tincidunt urna vel molestie."));
    }

    #[test]
    fn test_unquoted_identifier() {
        // Basic alphanumeric
        assert_eq!(
            parse_identifier("user123 did something"),
            Some(("user123", " did something"))
        );
        // Leading padding
        assert_eq!(
            parse_identifier("\r\n     user987 did something"),
            Some(("user987", " did something"))
        );
        // With special characters
        assert_eq!(
            parse_identifier("my-var_name.v1+ is an identifier"),
            Some(("my-var_name.v1+", " is an identifier"))
        );
        // Stops at disallowed characters
        assert_eq!(parse_identifier("field=next"), Some(("field", "=next")));
        // No further text
        assert_eq!(parse_identifier("ident"), Some(("ident", "")));
    }

    #[test]
    fn test_quoted_identifier() {
        // Basic quoted string
        assert_eq!(
            parse_identifier("\"a quoted id\" is cool"),
            Some(("a quoted id", " is cool"))
        );
        // Leading padding
        assert_eq!(
            parse_identifier("\t   \"another quoted id\" is also cool"),
            Some(("another quoted id", " is also cool"))
        );
        // No further text
        assert_eq!(parse_identifier("\"ident\""), Some(("ident", "")));
    }

    #[test]
    fn test_failures_and_edge_cases() {
        // Empty input
        assert_eq!(parse_identifier(""), None);
        assert_eq!(parse_identifier("   "), None);
        assert_eq!(parse_identifier("  =something"), None);
        // Unclosed quotes
        assert_eq!(parse_identifier("\"unfinished quote"), None);
    }
}

#[cfg(test)]
mod full_tests {
    use super::*;

    #[test]
    fn test_parsing_root_simple() {
        let root = "
chapter AFP

session \"AOT\" = \"HOL-Cardinals\" +
  options [show_question_marks = false, timeout = 600, names_short = true]
  sessions
    \"HOL-Cardinals\"
    \"HOL-Eisbach\"
  theories
    AOT_model
    AOT_commands
    AOT_syntax
    AOT_semantics
    AOT_Definitions
    AOT_Axioms
    AOT_PLM
    AOT_BasicLogicalObjects
    AOT_RestrictedVariables
    AOT_ExtendedRelationComprehension
    AOT_PossibleWorlds
    AOT_NaturalNumbers
    AOT_misc
  document_files
    \"root.tex\"

";
        // https://www.isa-afp.org/entries/AOT.html - Daniel Kirchner

        let parsed = parse_root(root);
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();

        assert_eq!(parsed.len(), 1);
        let session = &parsed[0];

        assert_eq!(session.name, "AOT");
        assert_eq!(session.parent, "HOL-Cardinals");
        assert_eq!(session.sessions, vec!["HOL-Cardinals", "HOL-Eisbach"]);
    }

    #[test]
    fn test_parsing_root_commented() {
        let root = "
chapter AFP

session Auto2_Imperative_HOL = Auto2_HOL +
  description \\<open>
    Application of auto2 to verify functional and imperative programs.
  \\<close>
  options [timeout = 2100]
  sessions
    \"HOL-Library\"
    \"HOL-Imperative_HOL\"
  directories
    \"Functional\"
    \"Imperative\"
  theories
    (* Functional programs *)
    \"Functional/BST\"
    \"Functional/Lists_Ex\"
    \"Functional/Connectivity\"
    \"Functional/Dijkstra\"
    \"Functional/Interval_Tree\"
    \"Functional/Quicksort\"
    \"Functional/Indexed_PQueue\"
    \"Functional/RBTree\"
    \"Functional/Rect_Intersect\"

    (* Imperative programs *)
    \"Imperative/GCD_Impl\"
    \"Imperative/LinkedList\"
    \"Imperative/BST_Impl\"
    \"Imperative/RBTree_Impl\"
    \"Imperative/Quicksort_Impl\"
    \"Imperative/Connectivity_Impl\"
    \"Imperative/Dijkstra_Impl\"
    \"Imperative/Rect_Intersect_Impl\"

  theories [document = false]
    \"Imperative/Sep_Examples\"

  document_files
    \"root.tex\"
    \"root.bib\"

";
        // https://www.isa-afp.org/entries/Auto2_Imperative_HOL.html - Bohua Zhan

        let parsed = parse_root(root);
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();

        assert_eq!(parsed.len(), 1);
        let session = &parsed[0];

        assert_eq!(session.name, "Auto2_Imperative_HOL");
        assert_eq!(session.parent, "Auto2_HOL");
        assert_eq!(session.sessions, vec!["HOL-Library", "HOL-Imperative_HOL"]);
    }

    #[test]
    fn test_parsing_root_complex() {
        let root = "
(*
 * Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
 * Copyright (c) 2022 Apple Inc. All rights reserved.
 *
 * SPDX-License-Identifier: BSD-2-Clause
 *
 *)

chapter AFP

session AutoCorres2 = Simpl +
  options [timeout = 3600]
  sessions
    \"Word_Lib\"
    \"HOL-Library\"
    \"HOL-Eisbach\"
    \"HOL-ex\"
  directories
    \"lib\"
    \"lib/subgoal_focus\"
    \"lib/ml-helpers\"
    \"lib/Monad_WP\"
    \"lib/clib\"
    \"c-parser\"
    \"c-parser/umm_heap\"
    \"c-parser/umm_heap/ARM\"
    \"c-parser/umm_heap/ARM64\"
    \"c-parser/umm_heap/ARM_HYP\"
    \"c-parser/umm_heap/RISCV64\"
    \"c-parser/umm_heap/X64\"

    \"doc\"
    \"doc/quickstart\"
  theories
    (* some libraries appear explicitly here (although they are subsequently imported) to
       tune the presentation sequence of the theories in the generated pdf document *)
    (* Library *)
    More_Lib
    \"MkTermAntiquote\"
    \"MkTermAntiquote_Tests\"
    \"TermPatternAntiquote\"
    \"TermPatternAntiquote_Tests\"

    \"Match_Cterm\"

    ML_Record_Antiquotation

    \"ML_Fun_Cache\"
    \"Tuple_Tools\"
    \"Subgoal_Methods\"
    \"Synthesize\"
    Rule_By_Method

    Option_Scanner
    Misc_Antiquotation    
    Runs_To_VCG
    Eisbach_Methods
    \"Option_MonadND\"
    \"Reader_Monad\"
    \"Apply_Trace_Cmd\"

    Tagging

    \"Mutual_CCPO_Recursion\"

    (* C-Parser *)
    \"CTranslation\"
    LemmaBucket_C
    TypHeapLib

    (* AutoCorres *)
    \"AutoCorres\"

    (* Documentation *)
    \"Chapter1_MinMax\"
    \"Chapter2_HoareHeap\"
    \"Chapter3_HoareHeap\"

    \"AutoCorres_Documentation\"
    \"CTranslationInfrastructure\"

  document_files
    \"root.bib\"
    \"root.tex\"

  document_files (in \"doc/quickstart/sources\")
    \"minmax.c\"
    \"mult_by_add.c\"
    \"swap.c\"

  document_files (in \"c-parser/doc\")
    \"ctranslation_body.tex\"
    \"ctranslation.bib\"

session AutoCorres2_Main in main = Simpl +
  options [timeout = 2400]
  sessions
    AutoCorres2 \\<comment> \\<open>not the parent session to avoid importing the doc / example theories\\<close>
  theories
    AutoCorres_Main
    AutoCorres_Nondet_Syntax

session AutoCorres2_Test in tests = AutoCorres2_Main +  
  options [timeout = 6000]
  sessions
    \"HOL-Number_Theory\"
  directories
    \"examples\"
    \"parse-tests\"
    \"proof-tests\"
    \"c-parser\"
    \"c-parser/includes\"
  theories
    \"CParserTest\"
    \"AutoCorresTest\"


";
        // https://www.isa-afp.org/entries/AutoCorres2.html - Matthew Brecknell, David Greenaway, Johannes Hölzl, Fabian Immler, Gerwin Klein, Rafal Kolanski, Japheth Lim, Michael Norrish, Norbert Schirmer, Salomon Sickert, Thomas Sewell, Harvey Tuch and Simon Wimmer

        let parsed = parse_root(root);
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();

        assert_eq!(parsed.len(), 3);
        let session1 = &parsed[0];
        let session2 = &parsed[1];
        let session3 = &parsed[2];

        assert_eq!(session1.name, "AutoCorres2");
        assert_eq!(session1.parent, "Simpl");
        assert_eq!(
            session1.sessions,
            vec!["Word_Lib", "HOL-Library", "HOL-Eisbach", "HOL-ex"]
        );

        assert_eq!(session2.name, "AutoCorres2_Main");
        assert_eq!(session2.parent, "Simpl");
        assert_eq!(session2.sessions, vec!["AutoCorres2"]);

        assert_eq!(session3.name, "AutoCorres2_Test");
        assert_eq!(session3.parent, "AutoCorres2_Main");
        assert_eq!(session3.sessions, vec!["HOL-Number_Theory"]);
    }
}
