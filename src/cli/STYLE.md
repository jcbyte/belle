# CLI Style Guide

## Colours

| Element | Color  | Intent                                           |
| ------- | ------ | ------------------------------------------------ |
| Success | Green  | Success action i.e. "Switched" prefix.           |
| Warning | Yellow | Non-fatal errors, potential mistakes.            |
| Error   | Red    | Fatal errors, unrecoverable/unresolvable.        |
| Focus   | Cyan   | Highlight of line i.e. environment/package name. |

## Prefixing

Each permanent line should use a bold, capitalised and colored verb at the start of lines.

- **Switched** to...
- **Migrated** environment to...

The status verb should be right-aligned with a fixed width for a clean vertical line.

### Errors & Warnings

Errors and warnings should follow the same suit with prefix listed as always "Error" or "Warning" respectively.

Traces must follow standard indentation level (for each trace).

## Grammar

Full stops should never be placed at the end of lines.

They may be placed within the same line, for multiple sentences.

## Summary Blocks

Commands with lists/long outputs should print a summary line giving ideally numerical output of changes.

Numerical data in summary commands should be bold.

## Progress Bars

A standardised style and colour theme should be used for progress bars and spinners across the application.

## Packages and Versions

Version numbers should be printed using square brackets ([2019.2.17]).

If the version number is explicit it should be fully coloured, else dimmed.

When referencing an AFP/Isabelle the given Isabelle name should also be used (2025-2 [2025.2.0]).

## No-Ops

When nothing needs to be changed (i.e. switching into the current environment) a dimmed message should be presented to indicate no action was taken.

## Lists

Lists should use the same indentation as regular commands with the prefix used sparingly to mark certain elements in cyan.

## Interactivity

No commands should require input once requested. They must only give status updates and succeed/fail.
