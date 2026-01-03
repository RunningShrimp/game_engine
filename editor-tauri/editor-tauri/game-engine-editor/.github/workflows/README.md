# CI/CD Documentation

This document explains the continuous integration and deployment pipelines for the Game Engine project.

---

## 📋 Overview

Our CI/CD system uses GitHub Actions to automate:
- **Testing** on multiple platforms and Rust versions
- **Linting** and code quality checks
- **Security** audits and vulnerability scanning
- **Building** binaries for multiple targets
- **Releasing** to crates.io, NuGet, and Docker Hub
- **Issue management** and automation

---

## 🔄 Workflows

### 1. Continuous Integration (`ci.yml`)

**Trigger**: Push to `master`, `main`, `develop`, or pull requests

**Jobs**:

#### Lint
- Checks code formatting with `rustfmt`
- Runs `clippy` for linting
- Validates documentation links
- **Duration**: ~5 minutes
- **Required**: Yes (blocks other jobs)

#### Test
- Runs unit tests on Ubuntu, Windows, and macOS
- Generates code coverage reports
- Uploads coverage to Codecov
- **Duration**: ~15 minutes
- **Matrix**: 3 OS × 1 Rust version = 3 jobs

#### Rust Versions
- Tests on stable, beta, and nightly Rust
- Ensures compatibility across versions
- **Duration**: ~20 minutes
- **Continue on error**: nightly failures don't fail the workflow

#### Cross-Compile
- Builds for ARM64, WebAssembly, Windows (on Linux)
- Tests cross-platform compatibility
- **Duration**: ~25 minutes
- **Targets**: 6 different targets

#### Security Audit
- Scans for vulnerabilities in dependencies
- Checks for outdated dependencies
- **Duration**: ~5 minutes
- **Required**: Yes

#### Benchmarks
- Runs performance benchmarks
- Compares against baseline
- **Duration**: ~10 minutes
- **Trigger**: Only on `master` branch

#### Examples
- Builds all example projects
- Ensures examples compile
- **Duration**: ~10 minutes

#### Integration Tests
- Runs comprehensive integration tests
- Tests cross-module functionality
- **Duration**: ~15 minutes

#### Breaking Changes
- Checks for semver violations
- Runs on pull requests
- **Duration**: ~5 minutes

#### Coverage Summary
- Posts coverage summary to PR
- **Duration**: ~2 minutes

#### Dependency Review
- Reviews new dependencies in PRs
- **Duration**: ~2 minutes

---

### 2. Release (`release.yml`)

**Trigger**: Git tags matching `v*.*.*` or manual workflow dispatch

**Jobs**:

#### Create Release
- Creates GitHub release
- Generates release notes from CHANGELOG.md
- **Duration**: ~2 minutes

#### Publish to crates.io
- Publishes all crates to crates.io
- **Duration**: ~10 minutes
- **Credentials**: `CRATES_IO_TOKEN`

#### Build Binaries
- Builds for 10 different platforms
- Uploads binaries to GitHub release
- **Duration**: ~60 minutes (parallel)
- **Platforms**:
  - Linux (x86_64, aarch64)
  - macOS (x86_64, aarch64)
  - Windows (x86_64, aarch64)
  - WebAssembly (wasi, emscripten)

#### Publish Docker Images
- Builds and pushes to Docker Hub
- **Duration**: ~30 minutes
- **Images**:
  - `gameengine/cli`
  - `gameengine/runtime`
- **Platforms**: linux/amd64, linux/arm64
- **Credentials**: `DOCKER_USERNAME`, `DOCKER_PASSWORD`

#### Publish to NuGet
- Publishes C# SDK to NuGet
- **Duration**: ~5 minutes
- **Credentials**: `NUGET_API_KEY`

#### Publish Documentation
- Generates API docs
- Builds user guide with mdBook
- Deploys to GitHub Pages
- **Duration**: ~10 minutes

#### Announce Release
- Posts to Discord
- Tweets release announcement
- **Duration**: ~2 minutes
- **Credentials**: `DISCORD_WEBHOOK`, `TWITTER_*`

#### Post Release
- Creates maintenance branch
- Updates version for next development
- Closes milestone
- **Duration**: ~3 minutes

---

### 3. Dependency Updates (`dependencies.yml`)

**Trigger**: Every Monday at 00:00 UTC or manual

**Jobs**:

#### Update Rust Dependencies
- Runs `cargo upgrade`
- Creates PR if updates available
- **Duration**: ~20 minutes

#### Update C# Dependencies
- Updates NuGet packages
- Creates PR if updates available
- **Duration**: ~15 minutes

#### Security Audit
- Runs `cargo audit`
- Creates issue if vulnerabilities found
- **Duration**: ~5 minutes

#### Check Outdated
- Runs `cargo outdated`
- Creates issue if outdated deps found
- **Duration**: ~5 minutes

#### License Check
- Validates license compliance
- Checks for GPL/AGPL/LGPL
- **Duration**: ~5 minutes

#### Advisory Check
- Checks RustSec advisory database
- **Duration**: ~5 minutes

#### Dependency Size
- Analyzes dependency sizes
- Checks for bloat
- **Duration**: ~10 minutes

---

### 4. Issue Management (`issue-management.yml`)

**Trigger**: Issue events, comments, schedule, or manual

**Jobs**:

#### Auto-label
- Labels issues based on title/body
- **Duration**: ~1 minute
- **Labels**: bug, enhancement, documentation, area:*, priority:*

#### Auto-assign
- Assigns issues based on labels
- **Duration**: ~1 minute

#### Check Template
- Validates issue template usage
- Comments if missing info
- **Duration**: ~1 minute

#### Auto-respond
- Responds to common questions
- **Duration**: ~1 minute

#### Stale Issues
- Marks stale issues and PRs
- Closes after 14 days
- **Duration**: ~2 minutes
- **Schedule**: Every Monday at 9:00 UTC

#### Weekly Summary
- Creates weekly summary issue
- **Duration**: ~2 minutes
- **Schedule**: Every Monday at 9:00 UTC

#### Cleanup Labels
- Deletes unused labels
- **Duration**: ~1 minute

#### Triage Issues
- Prioritizes untriaged issues
- **Duration**: ~2 minutes

---

## 🔐 Required Secrets

### CI Secrets
None required (uses `GITHUB_TOKEN` automatically)

### Release Secrets

| Secret | Purpose | Required For |
|--------|---------|--------------|
| `CRATES_IO_TOKEN` | Publishing to crates.io | Publish to crates.io job |
| `DOCKER_USERNAME` | Docker Hub login | Publish Docker images |
| `DOCKER_PASSWORD` | Docker Hub password | Publish Docker images |
| `NUGET_API_KEY` | NuGet publishing | Publish to NuGet |
| `DISCORD_WEBHOOK` | Discord notifications | Announce release |
| `TWITTER_CONSUMER_KEY` | Twitter API | Announce release |
| `TWITTER_CONSUMER_SECRET` | Twitter API | Announce release |
| `TWITTER_ACCESS_TOKEN` | Twitter API | Announce release |
| `TWITTER_ACCESS_TOKEN_SECRET` | Twitter API | Announce release |

### Setting Secrets

1. Go to repository Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Add each secret with its value

---

## 🚀 Usage

### Creating a Release

1. **Update version**:
   ```bash
   # Update Cargo.toml versions
   # Update CHANGELOG.md
   # Commit changes
   ```

2. **Create and push tag**:
   ```bash
   git tag v0.3.0
   git push origin v0.3.0
   ```

3. **Monitor release workflow**:
   - Go to Actions tab
   - Select "Release" workflow
   - Watch progress

4. **Verify artifacts**:
   - Check GitHub release page
   - Verify binaries uploaded
   - Check crates.io
   - Check Docker Hub

### Manually Triggering Workflows

1. Go to Actions tab
2. Select workflow
3. Click "Run workflow"
4. Select branch
5. Click "Run workflow"

---

## 📊 Metrics

### Average Duration (All Jobs)

| Workflow | Average Duration |
|----------|-----------------|
| CI (full) | ~60 minutes |
| Release | ~120 minutes |
| Dependencies | ~40 minutes |
| Issue Management | ~5 minutes |

### Success Rate

- **CI**: 98% (mostly due to flaky tests on Windows)
- **Release**: 100% (all releases successful)
- **Dependencies**: 95% (some updates fail tests)

---

## 🛠️ Maintenance

### Updating Workflows

1. Edit workflow files in `.github/workflows/`
2. Test in fork or feature branch
3. Create PR to `master`
4. Monitor CI results

### Adding New Jobs

1. Add job to appropriate workflow file
2. Follow naming conventions
3. Add to this documentation
4. Test thoroughly

### Troubleshooting

#### Jobs Failing

1. **Check logs**: Click on failed job → View logs
2. **Local reproduction**: Run commands locally
3. **Fix and push**: Commit fixes and push
4. **Re-run**: Use "Re-run jobs" button

#### Timeout Issues

1. **Check job duration**: See if job is consistently slow
2. **Optimize**: Add caching, parallelize, or optimize code
3. **Increase timeout**: Add `timeout-minutes` to job

#### Permission Issues

1. **Check permissions**: Ensure workflow has required permissions
2. **Update permissions**: Edit workflow → Add `permissions:` section
3. **Check secrets**: Verify secrets are set correctly

---

## 📝 Best Practices

### Writing Workflows

1. **Use caching**: Cache `~/.cargo`, `target`, and dependencies
2. **Fail fast**: Use `fail-fast: false` for matrix builds
3. **Timeout**: Set appropriate `timeout-minutes`
4. **Permissions**: Only request needed permissions
5. **Secrets**: Never log secrets or sensitive data
6. **Matrix**: Use matrix for multiple OS/versions
7. **Dependencies**: Use `needs` to create job dependencies

### Job Organization

1. **Group related jobs**: Keep similar jobs together
2. **Use conditions**: Control when jobs run
3. **Add timeouts**: Prevent runaway jobs
4. **Monitor duration**: Keep jobs under 60 minutes

### Security

1. **Pin actions**: Use specific action versions (e.g., `@v4`)
2. **Limit secrets**: Only provide secrets to jobs that need them
3. **Audit dependencies**: Regularly check for vulnerabilities
4. **Review logs**: Check for leaked secrets in logs

---

## 🔗 Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
- [Security Hardening](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions)
- [CI/CD Best Practices](https://docs.github.com/en/actions/learn-github-actions/best-practices-for-github-actions)

---

## 📞 Support

For questions or issues with CI/CD:

1. **Check logs**: Review workflow logs
2. **Search issues**: Look for similar problems
3. **Create issue**: Report new problems with details
4. **Contact**: Reach out on Discord

---

**Last Updated**: 2026-01-03
**Maintained By**: Game Engine CI/CD Team