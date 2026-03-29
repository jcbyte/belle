# CLI Style Guide

## Colours

| Element | Color          | Intent                                           |
| ------- | -------------- | ------------------------------------------------ |
| Success | Green          | Success action i.e. "Switched" prefix.           |
| Warning | Yellow         | Non-fatal errors, potential mistakes.            |
| Error   | Red            | Fatal errors, unrecoverable/unresolvable.        |
| Focus   | Cyan (Bright)  | Highlight of line i.e. environment/package name. |
| No-op   | White (Dimmed) | For no operation commands.                       |

## Prefixing

Each permanent line should use a bold, capitalised and colored verb at the start of lines.

- **Switched** to...
- **Migrated** environment to...

The status verb should be right-aligned with a fixed width for a clean vertical line.

### Errors, Warnings & No-Ops

Errors, warnings and no-ops should follow the same suit with prefix listed as always "Error", "Warning" or "Skipped" respectively.

Traces must follow standard incrementing indentation level (for each trace).

## Grammar

Full stops should never be placed at the end of lines. They may be placed within the same line, for multiple sentences, however a semi-colon is preferred.

Capitals should be avoided within text, apart from prefixes.

## Summary Blocks

Commands with lists/long outputs should print a summary line giving ideally numerical output of changes.

Numerical data in summary commands should be bold.

## Packages and Versions

When printing packages, always print the name and version together where possible.

Version numbers should be printed with a space and prefixed with 'v': (Z_Toolkit v2019.2.17).

If the version number is implicit it should be dimmed.

When referencing an AFP/Isabelle the given Isabelle name should also be used, with no string prefix (i.e. stripping "afp-" from "afp-2025"): (Isabelle 2025-2 v2025.2.0, AFP 2025-2 v2025.2.0).

## Progress Bars

A standardised style and colour theme should be used for progress bars and spinners across the application.

## No-Ops

When nothing needs to be changed (i.e. switching into the current environment) a message should be presented to indicate no action was taken.

## Lists

Lists should use the same indentation as regular commands with the prefix used sparingly to mark certain elements in cyan.

## Interactivity

No commands should require input once requested. They must only give status updates and succeed/fail.
