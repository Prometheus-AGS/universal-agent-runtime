# dependabot-yml

## Why

`gh api repos/.../dependabot/alerts` showed 96 open alerts (5 critical,
17 high) with no automated update pipeline — `.github/dependabot.yml`
didn't exist. Alerts had been accumulating silently for ~4 months with
nobody surfacing them as reviewable diffs.

## What changed

New `.github/dependabot.yml`: 4 ecosystems configured (`cargo` at root,
`npm` at root, `npm` at `/frontend` — a separate pnpm project with its
own lockfile — and `github-actions`), weekly schedule, minor/patch
updates grouped per ecosystem to reduce PR noise, 10-PR cap per
ecosystem.

## Verification

- YAML syntax validated: `python3 -c "import yaml; yaml.safe_load(open('.github/dependabot.yml'))"`.
- Config-only change; no code touched, no test-suite checkpoint needed.
