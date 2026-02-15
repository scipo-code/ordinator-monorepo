# charlie-comment

Standardize comments in Rust (.rs) files across the monorepo.

## Instructions

Read through every `.rs` file in the codebase and standardize comments according to these guidelines:

1. Fix grammar, spelling, and punctuation errors while preserving the original technical meaning
2. Condense verbose or rambling comments into clear, concise statements without losing essential context
3. Remove redundant comments that merely restate what the code obviously does (e.g., `// increment i` above `i += 1`)
4. Ensure doc comments (`///` and `//!`) follow Rust conventions: start with a verb, describe what the item does, not how
5. Standardize formatting: single space after `//`, capitalize first word, no trailing periods for single-line comments
