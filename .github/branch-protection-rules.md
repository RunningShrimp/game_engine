# Branch Protection Rules for Game Engine

This document describes the branch protection rules configured for the game engine repository to ensure code quality and stability.

## Protected Branches

### Main Branch (`main` / `master`)

The main branch is protected with the following rules:

#### 1. Status Checks Required

All of the following status checks must pass before merging:

**Critical Checks (Must Pass):**
- `format` - Code formatting check
- `clippy` - Clippy linting
- `test` - Unit and integration tests
- `doc` - Documentation tests and coverage
- `coverage` - Code coverage threshold (>= 50%)

**Quality Checks:**
- `audit` - Security audit
- `udeps` - Unused dependencies check
- `outdated` - Outdated dependencies check
- `complexity` - Code complexity analysis
- `clippy-dynamic` - Clippy suggestions generator
- `performance-regression` - Performance regression detection
- `duplicate-code` - Duplicate code detection
- `style-check` - Code style consistency

**Build Checks:**
- `examples` - Build examples verification

#### 2. Branch Protection Settings

- **Require pull request reviews before merging**: Enabled
  - Required approving reviews: 1
  - Dismiss stale reviews when new commits are pushed: Enabled
  - Require review from code owners: Enabled

- **Require status checks to pass before merging**: Enabled
  - Require branches to be up to date before merging: Enabled

- **Restrict who can push to matching branches**:
  - Only allow: `admin`, `maintainer`

- **Allow force pushes**: Disabled

- **Allow deletions**: Disabled

#### 3. Quality Gates

**Documentation Coverage Gate:**
- Minimum coverage: 90%
- Blocks merge if below threshold

**Test Coverage Gate:**
- Minimum coverage: 50%
- Warning if below 70%
- Blocks merge if below 50%

**Performance Regression Gate:**
- Threshold: 5% regression
- Blocks merge if performance degrades more than 5%
- Compares against main branch baseline

**Security Gate:**
- Blocks merge if critical vulnerabilities found
- Warning for medium/high vulnerabilities
- Requires manual review for security issues

#### 4. Auto-merge Rules

Auto-merge is enabled for trusted contributors when:
- All critical checks pass
- No security vulnerabilities
- No performance regression
- Documentation coverage >= 90%
- Code coverage >= 70%

### Development Branch (`develop`)

The develop branch has relaxed rules:

- **Require status checks**: Only critical checks (format, clippy, test)
- **Require reviews**: Optional
- **Allow force pushes**: Enabled for admins

## Setting Up Branch Protection

### Using GitHub UI

1. Go to repository Settings
2. Click on "Branches" in the left sidebar
3. Click "Add rule" or edit existing rule for `main`
4. Configure settings as described above
5. Save changes

### Using GitHub CLI

```bash
# Enable branch protection for main
gh api repos/:owner/:repo/branches/main/protection \
  --method PUT \
  -f required_status_checks='{"strict":true,"contexts":["format","clippy","test","doc","coverage"]}' \
  -f enforce_admins=true \
  -f required_pull_request_reviews='{"required_approving_review_count":1}' \
  -f restrictions=null
```

### Using Terraform

```hcl
resource "github_branch_protection" "main" {
  repository_id = github_repository.game_engine.node_id
  branch        = "main"

  required_status_checks {
    strict   = true
    contexts = ["format", "clippy", "test", "doc", "coverage"]
  }

  required_pull_request_reviews {
    required_approving_review_count = 1
    dismiss_stale_reviews          = true
    require_code_owner_reviews     = true
  }

  enforce_admins = true
}
```

## Status Check Configuration

### Quality Gate Workflow

The main quality gate workflow runs on:
- Push to `main` or `develop`
- Pull requests to `main` or `develop`
- Manual workflow dispatch

### Check Results

All check results are:
- Displayed in the PR summary
- Uploaded as artifacts (90-day retention)
- Available in GitHub Actions summary
- Reported as comments on PRs

## Bypassing Rules

In exceptional circumstances, maintainers can bypass branch protection by:

1. Using the "Dismiss reviews" button (with justification)
2. Temporarily relaxing rules (document the reason)
3. Using admin override (requires documented approval)

All bypasses must be:
- Documented in the commit message
- Reviewed by another maintainer
- Reported in the next team meeting

## Monitoring and Alerts

### Daily Reports

- Quality gate pass/fail rates
- Average time to pass checks
- Common failure reasons

### Weekly Reports

- Coverage trends
- Performance trends
- Security vulnerability trends

### Alerts

Immediate alerts for:
- Critical security vulnerabilities
- Performance regressions > 10%
- Test failures in main branch
- Documentation coverage drops

## Related Documentation

- [Quality Gate Guide](/docs/CI_CD_QUALITY_GATE_GUIDE.md)
- [Testing Guide](/docs/testing_guide.md)
- [Performance Monitoring](/docs/performance_optimization_P0-4.md)
- [Security Best Practices](/docs/code-quality/security.md)
