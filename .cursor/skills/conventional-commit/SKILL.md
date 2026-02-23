---
name: conventional-commit
description: Generate conventional commit messages from staged git changes for the user to copy. Use when the user wants a commit message, asks to commit, or wants to review staged changes for committing.
---

# Conventional Commit

Generate commit messages following [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/).

## Workflow

1. Run `git diff --cached` to get staged changes. If nothing is staged, tell the user "No staged changes found, aborting." and stop.

2. Analyze the diff and generate 3 commit message candidates following the format and rules below.

3. Print the 3 candidates as copyable code blocks so the user can pick and paste one into their terminal. Do NOT run `git commit` or any git write command — only print the messages.

## Format Rules

- `<type>[optional scope]: <description>`
- Description: lowercase, imperative mood, no trailing period.
- Subject line under 72 characters.
- Scope is optional, in parentheses: `feat(cpu): add temperature parsing`.
- Breaking changes: `!` after type/scope and/or `BREAKING CHANGE:` footer.

## Allowed Types

`feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`

## Examples

```
feat(auth): implement JWT-based authentication
```

```
fix(reports): correct date formatting in timezone conversion
```

```
docs: update API reference for v2 endpoints
```
