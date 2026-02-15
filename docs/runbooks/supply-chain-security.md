# Supply Chain Security Runbook

This runbook covers the `Security Supply Chain Guardrails` GitHub Actions workflow in `.github/workflows/security-supply-chain.yml`.

## What this workflow checks

On every pull request (and manual dispatch), the workflow runs three guardrails:

1. **SBOM generation**
   - Generates a CycloneDX JSON SBOM for the repository.
   - Uploads it as a workflow artifact named `cyclonedx-sbom`.

2. **Vulnerability scan**
   - Runs Trivy filesystem scan against the repository.
   - Fails the workflow when vulnerabilities of severity `HIGH` or `CRITICAL` are found.

3. **Generated artifact commit guard**
   - Computes the PR diff file list (`base.sha` to `head.sha`).
   - Executes `scripts/security/artifact_guard.sh` to fail if any forbidden generated artifacts are present in the diff.

Forbidden patterns:

- `**/node_modules/**`
- `**/.next/**`
- `**/dist/**`
- `**/__pycache__/**`
- `*.pyc`

## Triage guide for failures

### 1) SBOM step failure

- Confirm the job has read access to repository contents.
- Re-run the workflow to rule out transient action/network failures.
- If persistent, pin/update the SBOM action version in the workflow and note the change in PR.

### 2) Vulnerability scan failure (Trivy)

- Open the Trivy log in workflow output and identify each `HIGH`/`CRITICAL` finding.
- Determine whether the vulnerable component is:
  - direct dependency,
  - transitive dependency,
  - or false positive.
- Preferred resolution order:
  1. Upgrade vulnerable package/dependency to a fixed version.
  2. Remove/replace the vulnerable dependency.
  3. If no fix exists, document risk and temporary mitigation in the PR description and security tracker.

### 3) Artifact guard failure

- Review file paths listed by `artifact_guard.sh` in workflow logs.
- Remove generated artifacts from Git index, for example:
  - `git rm -r --cached path/to/node_modules`
  - `git rm -r --cached path/to/dist`
- Ensure `.gitignore` is configured so those artifacts are not re-added.
- Re-commit and push the PR branch.

## Exceptions and approvals

Exceptions should be rare and time-bounded.

1. **Request**: PR author documents why an exception is required, scope, and expiration date.
2. **Approval**: Requires explicit approval from:
   - **Security owner** (or delegate), and
   - **Code owner** for the affected area.
3. **Record**: Link approval in PR conversation and create a follow-up task to remove exception.
4. **Implementation**: Exception handling must be explicit and narrow (single path/package/CVE), never a broad disable of the workflow.

If approval is not present, the workflow failure is blocking and must be fixed before merge.
