# CI: GKE build-and-deploy — required GitHub repo secrets

The **Build and Deploy to GKE** workflow (`.github/workflows/deploy.yml`) builds the
runtime Docker image and rolls the `uar` Deployment on the GKE `client-cluster`. It
failed on `main` because the required GitHub **repo secrets are not configured** —
`google-github-actions/auth@v2` reported:

> the GitHub Action workflow must specify exactly one of "workload_identity_provider"
> or "credentials_json" … ensure the secret is being injected

and `PROJECT_ID` rendered empty. The workflow YAML is correct; it just needs the
secrets below set on **`Prometheus-AGS/universal-agent-runtime` → Settings → Secrets
and variables → Actions**.

## Required secrets

| Secret | Used at | What it is |
| --- | --- | --- |
| `GCP_PROJECT_ID` | `deploy.yml` env `PROJECT_ID` | `prometheus-461323` |
| `WIF_PROVIDER` | `google-github-actions/auth` | the Workload Identity Federation provider resource name, e.g. `projects/<NUM>/locations/global/workloadIdentityPools/<POOL>/providers/<PROVIDER>` |
| `WIF_SERVICE_ACCOUNT` | `google-github-actions/auth` | the GCP service account email the WIF provider impersonates, e.g. `gha-deployer@prometheus-461323.iam.gserviceaccount.com` |
| `GKE_ZONE` *(optional)* | env `GKE_ZONE` | defaults to `us-central1` if unset |

The service account needs: Artifact Registry **writer** (push to
`us-docker.pkg.dev/prometheus-461323/uar/...`) + GKE **developer**
(`container.clusters.get` + apply on the `uar` namespace).

## Alternative (simpler, less secure): credentials_json
If Workload Identity Federation isn't set up, swap the auth step in `deploy.yml` to a
service-account key:
```yaml
- uses: google-github-actions/auth@v2
  with:
    credentials_json: ${{ secrets.GCP_SA_KEY }}
```
and add `GCP_SA_KEY` (the JSON key) as a repo secret. WIF is preferred (no long-lived key).

## Other CI fixes in this PR (no secrets needed)
- `ci.yml`: `dtolnay/rust-action@stable` → **`dtolnay/rust-toolchain@stable`** (the former
  action does not exist → "repository not found").
- `ci.yml`, `deploy.yml`, `comprehensive-tests.yml`, `quick-tests.yml`: added
  **`submodules: recursive`** to every `actions/checkout` so the git submodules
  (`crates/prometheus-skill-system`, `frontend/packages/prometheus-entity-management`,
  `models.dev`, …) are present — without them the frontend type-check fails
  (`Cannot find module '@prometheus-ags/prometheus-entity-management'`) and the Rust
  build can't see the skill-system crate.

Once the secrets are set, push to `main` (or run the workflow via **Actions →
Build and Deploy to GKE → Run workflow**) to build + deploy the current `main`.
