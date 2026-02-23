---
name: update-screenshot
description: Replace the project screenshot with a newly uploaded file. Use when the user mentions updating, replacing, or refreshing the screenshot, or uploads a new screenshot to the docs folder.
---

# Update Screenshot

## Instructions

1. Run `git status --porcelain docs/` and look for **untracked** files (lines starting with `??`) that match common image extensions (`.png`, `.jpg`, `.jpeg`, `.webp`).
2. If exactly one untracked image is found, copy it over `docs/ttop.png` and delete it.
3. If multiple untracked images are found, ask the user which one to use.
4. If none are found, tell the user no new screenshot was detected.

```bash
cp "docs/<new-file>" docs/ttop.png
rm "docs/<new-file>"
```
