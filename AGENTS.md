# Agent Instructions

Use this file as the default guide for AI agents working in the repository.

## Human In The Loop

- Keep the user informed when making substantial changes.
- Do not commit, push, deploy, or change shared infrastructure without explicit approval.
- Do not modify unrelated user changes in the worktree.

## Quick Rules

- Inspect the source before changing behavior.
- No dead code is allowed. Remove it or comment it out. 
  - For example, `#[allow(dead_code)]` is not allowed.
- Keep changes minimal and localized.
- Follow existing code style and conventions.
- Do not rely on stale documentation when the source disagrees.
- Do not divert from the active task without being asked.
- When writing text, anywhere, do not have sentences go over multiple lines in an unclear format.
  - Prefer `Text here.` <br> `More text here...` over `Text here. More` <br> `text here...`
- Do not use em dashes in code, comments, or documentation.
  - Use singular hyphens instead.
  - In the case of double em dashes, use triple hyphens instead.
  - If you are editing documentation that already has em dashes, replace them with hyphens.

## Code Conventions

- Avoid blocking I/O in async code.
- Prefer explicit error handling over panics.
- Add comments only when they clarify non-obvious behavior.

## Finally

Thanks for your contributions <3, Love the support.