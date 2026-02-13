# Git Hooks

## Pre-commit: Architecture Drift Check

The `pre-commit` hook runs `archidoc --check` before each commit. If your architecture docs are out of sync with source annotations, the commit is blocked.

### Installation

Copy the hook to your local `.git/hooks/` directory:

```bash
cp hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Or point git at this directory directly:

```bash
git config core.hooksPath hooks
```

### Skipping the hook

If you need to commit without the drift check (e.g., work-in-progress):

```bash
git commit --no-verify -m "WIP: ..."
```

## Required Secrets (for CI/CD)

The publish workflow (`.github/workflows/publish.yml`) requires two repository secrets:

| Secret | Source | Used for |
|--------|--------|----------|
| `CARGO_REGISTRY_TOKEN` | [crates.io API tokens](https://crates.io/settings/tokens) | `cargo publish` |
| `NPM_TOKEN` | [npm access tokens](https://www.npmjs.com/settings/~/tokens) | `npm publish` |

Set these in your GitHub repository: Settings > Secrets and variables > Actions > New repository secret.
