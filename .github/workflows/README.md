# CI/CD Documentation

This directory contains all GitHub Actions workflows for the Tracker Platform. This document explains each pipeline, how to use them, and troubleshooting guides.

## Table of Contents

- [Overview](#overview)
- [Workflows](#workflows)
  - [CI Pipeline](#ci-pipeline)
  - [Load Testing](#load-testing)
  - [Docker Build & Push](#docker-build--push)
  - [Release Pipeline](#release-pipeline)
  - [Database Migrations](#database-migrations)
  - [Documentation](#documentation)
  - [Dependency Updates](#dependency-updates)
- [Secrets Configuration](#secrets-configuration)
- [Manual Workflows](#manual-workflows)
- [Troubleshooting](#troubleshooting)
- [Best Practices](#best-practices)

## Overview

Our CI/CD infrastructure is built on GitHub Actions and provides:

- **Automated Testing**: Unit, integration, and E2E tests on every push
- **Code Quality**: Linting, formatting, and security checks
- **Continuous Deployment**: Automated Docker builds and releases
- **Documentation**: Auto-generated and deployed documentation
- **Dependency Management**: Automated updates via Dependabot
- **Load Testing**: Performance testing and benchmarking

## Workflows

### CI Pipeline

**File**: `ci.yml`

**Triggers**:
- Push to `main`, `develop`, `feature/**`, `bugfix/**` branches
- Pull requests to `main`, `develop`

**Jobs**:

1. **Lint Rust** - Runs `cargo fmt --check` and `cargo clippy`
2. **Lint Frontend** - Runs ESLint, Prettier, and Svelte checks
3. **Build Rust** - Compiles the backend in release mode
4. **Build Frontend** - Builds the frontend with Vite
5. **Test Rust** - Runs unit and integration tests with coverage
6. **Test Frontend** - Runs frontend unit tests
7. **E2E Tests** - Runs Playwright end-to-end tests
8. **Security Audits** - Checks for vulnerabilities in Rust and NPM dependencies

**Features**:
- Caching for faster builds (Cargo, NPM)
- Parallel job execution
- Coverage reports uploaded to Codecov
- OWASP dependency scanning
- Test artifacts (screenshots, videos)

**Status Badge**:
```markdown
![CI Pipeline](https://github.com/YOUR_REPO/workflows/CI%20Pipeline/badge.svg)
```

### Load Testing

**File**: `load-test.yml`

**Triggers**:
- Manual dispatch (workflow_dispatch)
- Scheduled (Sundays at 2 AM UTC)

**Jobs**:

1. **Load Test** - Runs k6 load tests with configurable VUs and duration
2. **Stress Test** - Tests system under stress with increasing load

**Parameters** (Manual Trigger):
- `duration`: Test duration (default: 5m)
- `virtual_users`: Number of concurrent users (default: 100)
- `environment`: Target environment (staging/production)

**Usage**:
```bash
# Via GitHub CLI
gh workflow run load-test.yml \
  -f duration=10m \
  -f virtual_users=200 \
  -f environment=staging

# Via GitHub UI
Actions > Load Testing > Run workflow
```

**Outputs**:
- JSON test results
- HTML performance report
- Comparison with baseline
- GitHub Pages deployment

### Docker Build & Push

**File**: `docker.yml`

**Triggers**:
- Push to `main`, `develop`
- Tag push (`v*.*.*`)
- Pull requests to `main`
- Manual dispatch

**Jobs**:

1. **Build and Push** - Multi-arch Docker builds (amd64, arm64)
2. **Security Scan** - Trivy vulnerability scanning
3. **Update Docker Compose** - Updates image tags in docker-compose.yml
4. **Test Image** - Smoke tests for built images
5. **Sign Image** - Cosign image signing (for releases)

**Features**:
- Multi-architecture support (AMD64, ARM64)
- Layer caching for faster builds
- SARIF security reports
- Automatic image tagging (branch, SHA, semver)
- GitHub Container Registry integration

**Image Tags**:
- `latest` - Latest main branch build
- `main` - Main branch
- `develop` - Develop branch
- `v1.2.3` - Semver tags
- `sha-abc123` - Commit SHA

### Release Pipeline

**File**: `release.yml`

**Triggers**:
- Tag push matching `v*.*.*` (e.g., v1.0.0)
- Manual dispatch with version input

**Jobs**:

1. **Create Release** - Creates GitHub Release with changelog
2. **Build Binaries** - Compiles for multiple platforms:
   - Linux (amd64, arm64)
   - macOS (amd64, arm64)
   - Windows (amd64)
3. **Build Docker Images** - Tagged release images
4. **Package Release** - Creates installation package
5. **Deploy Staging** - Optional staging deployment
6. **Notify** - Discord/Slack notifications

**Creating a Release**:

```bash
# Tag and push
git tag v1.0.0
git push origin v1.0.0

# Or use GitHub CLI
gh release create v1.0.0 --generate-notes

# Or manual dispatch
gh workflow run release.yml -f version=v1.0.0
```

**Release Assets**:
- Platform-specific binaries (.tar.gz, .zip)
- Docker Compose package
- Installation scripts
- Documentation

### Database Migrations

**File**: `migrate.yml`

**Triggers**:
- Manual dispatch with environment and action selection
- Push to `main` affecting `tracker-platform/migrations/**`

**Actions**:

1. **Validate** - Checks migration file syntax and naming
2. **Run** - Executes migrations on target environment
3. **Rollback** - Provides rollback instructions

**Parameters**:
- `environment`: development/staging/production
- `action`: validate/run/rollback
- `steps`: Number of rollback steps

**Usage**:

```bash
# Validate migrations
gh workflow run migrate.yml \
  -f environment=staging \
  -f action=validate

# Run migrations on staging
gh workflow run migrate.yml \
  -f environment=staging \
  -f action=run

# Get rollback instructions
gh workflow run migrate.yml \
  -f environment=production \
  -f action=rollback \
  -f steps=1
```

**Safety Features**:
- Automatic backup before production migrations
- Multiple PostgreSQL version testing
- Migration validation checks
- Rollback capability
- Approval required for production

### Documentation

**File**: `docs.yml`

**Triggers**:
- Push to `main` affecting Rust code, Markdown, or frontend
- Pull requests to `main`
- Manual dispatch

**Jobs**:

1. **Build Rust Docs** - Generates rustdoc documentation
2. **Build API Docs** - Creates OpenAPI specification
3. **Build Frontend Docs** - Frontend component documentation
4. **Generate Badges** - README badge documentation
5. **Deploy Docs** - Publishes to GitHub Pages

**Documentation URLs**:
- Main docs: `https://YOUR_USERNAME.github.io/YOUR_REPO/`
- Rust API: `https://YOUR_USERNAME.github.io/YOUR_REPO/rust-docs/`
- REST API: `https://YOUR_USERNAME.github.io/YOUR_REPO/api-docs/`

**Manual Trigger**:
```bash
gh workflow run docs.yml
```

### Dependency Updates

**File**: `dependencies.yml`

**Triggers**:
- Scheduled (Mondays at 9 AM UTC)
- Manual dispatch

**Jobs**:

1. **Check Dependencies** - Lists outdated Rust and NPM packages
2. **Update Rust** - Creates PR for Rust dependency updates
3. **Update NPM** - Creates PR for NPM dependency updates
4. **Security Audit** - Checks for vulnerabilities
5. **Dependency Report** - Comprehensive status report

**Features**:
- Automatic PRs for patch updates
- Security vulnerability alerts
- Detailed dependency reports
- Auto-merge capability (configurable)

**Dependabot Integration**:

See `.github/dependabot.yml` for configuration. Dependabot handles:
- Weekly dependency updates
- Security vulnerability patches
- GitHub Actions updates
- Docker base image updates

## Secrets Configuration

Configure these secrets in **Settings > Secrets and variables > Actions**:

### Required Secrets

| Secret | Description | Used By |
|--------|-------------|---------|
| `GITHUB_TOKEN` | Automatically provided by GitHub | All workflows |

### Optional Secrets

| Secret | Description | Used By | Example |
|--------|-------------|---------|---------|
| `DISCORD_WEBHOOK` | Discord webhook for notifications | All workflows | `https://discord.com/api/webhooks/...` |
| `SLACK_WEBHOOK` | Slack webhook for notifications | Release, Migrations | `https://hooks.slack.com/services/...` |
| `PRODUCTION_DATABASE_URL` | Production database connection | Migrations | `postgresql://user:pass@host:5432/db` |
| `STAGING_DATABASE_URL` | Staging database connection | Migrations | `postgresql://user:pass@host:5432/db` |
| `DEVELOPMENT_DATABASE_URL` | Development database connection | Migrations | `postgresql://user:pass@host:5432/db` |
| `STAGING_DEPLOY_KEY` | SSH key for staging deployment | Release | SSH private key |
| `PROD_URL` | Production URL for load testing | Load Testing | `https://api.example.com` |
| `CODECOV_TOKEN` | Codecov upload token | CI | `abc123...` |

### Adding Secrets

**Via GitHub UI**:
1. Go to Settings > Secrets and variables > Actions
2. Click "New repository secret"
3. Enter name and value
4. Click "Add secret"

**Via GitHub CLI**:
```bash
# Set a secret
gh secret set DISCORD_WEBHOOK

# Set from file
gh secret set STAGING_DEPLOY_KEY < ~/.ssh/deploy_key

# List secrets
gh secret list
```

## Manual Workflows

### Running Manual Workflows

**Via GitHub UI**:
1. Go to Actions tab
2. Select workflow from left sidebar
3. Click "Run workflow" button
4. Fill in parameters
5. Click "Run workflow"

**Via GitHub CLI**:
```bash
# Load testing
gh workflow run load-test.yml \
  -f duration=10m \
  -f virtual_users=200 \
  -f environment=staging

# Database migration
gh workflow run migrate.yml \
  -f environment=staging \
  -f action=run

# Release
gh workflow run release.yml \
  -f version=v1.0.0

# Documentation
gh workflow run docs.yml

# Dependency check
gh workflow run dependencies.yml
```

### Viewing Workflow Runs

```bash
# List recent runs
gh run list --workflow=ci.yml --limit 10

# View specific run
gh run view <run-id>

# Watch a running workflow
gh run watch

# View logs
gh run view <run-id> --log

# Download artifacts
gh run download <run-id>
```

## Troubleshooting

### Common Issues

#### 1. Build Failures

**Rust compilation errors**:
```bash
# Check locally
cd tracker-platform
cargo check --all-features

# Clear cache if needed
cargo clean
```

**Frontend build errors**:
```bash
# Check locally
cd tracker-platform/frontend
npm ci
npm run build
```

#### 2. Test Failures

**Database connection issues**:
- Ensure PostgreSQL service is healthy
- Check DATABASE_URL environment variable
- Verify migrations are up to date

**Redis connection issues**:
- Check Redis service status
- Verify REDIS_URL configuration

#### 3. Docker Build Issues

**Multi-arch build failures**:
```bash
# Test locally with Docker Buildx
docker buildx build --platform linux/amd64,linux/arm64 .
```

**Layer caching issues**:
- Clear GitHub Actions cache
- Settings > Actions > Caches > Delete old caches

#### 4. Migration Issues

**Migration validation fails**:
- Check migration file naming: `YYYYMMDDHHMMSS_description.sql`
- Verify SQL syntax
- Test locally with sqlx-cli

**Migration execution fails**:
- Check database permissions
- Verify network connectivity
- Review migration order

#### 5. Secret Issues

**Missing secrets**:
```bash
# List configured secrets
gh secret list

# Set missing secret
gh secret set SECRET_NAME
```

### Debug Mode

Enable debug logging in workflows:

```yaml
env:
  ACTIONS_STEP_DEBUG: true
  ACTIONS_RUNNER_DEBUG: true
```

Or via repository settings:
- Settings > Secrets > New secret
- Name: `ACTIONS_STEP_DEBUG`
- Value: `true`

### Re-running Failed Jobs

**Via GitHub UI**:
1. Go to Actions > Failed workflow
2. Click "Re-run failed jobs"

**Via GitHub CLI**:
```bash
gh run rerun <run-id>

# Re-run only failed jobs
gh run rerun <run-id> --failed
```

## Best Practices

### 1. Branch Protection

Enable branch protection for `main`:
- Settings > Branches > Add rule
- Branch name pattern: `main`
- Required status checks:
  - CI Pipeline / lint-rust
  - CI Pipeline / lint-frontend
  - CI Pipeline / test-rust
  - CI Pipeline / build-rust
- Require pull request reviews
- Require linear history

### 2. Workflow Optimization

**Use caching**:
```yaml
- uses: actions/cache@v4
  with:
    path: target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

**Parallel jobs**:
- Use `needs: []` to run jobs in parallel
- Only add dependencies when necessary

**Matrix builds**:
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
```

### 3. Security

**Secrets management**:
- Never commit secrets to repository
- Use GitHub Secrets for sensitive data
- Rotate secrets regularly
- Use least privilege principle

**Dependency security**:
- Enable Dependabot
- Review security advisories
- Run security audits regularly
- Keep dependencies updated

### 4. Testing

**Test locally before pushing**:
```bash
# Run lints
cargo fmt --check
cargo clippy

# Run tests
cargo test --all-features

# Check frontend
cd frontend && npm run check && npm run lint
```

**Use act for local testing**:
```bash
# Install act
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Run workflow locally
act push
act pull_request
```

### 5. Documentation

**Keep documentation updated**:
- Update this README when adding workflows
- Document workflow parameters
- Add troubleshooting guides
- Include examples

**Use workflow descriptions**:
```yaml
name: CI Pipeline
on: [push, pull_request]

# Add description
# This workflow runs on every push and PR
```

## Monitoring and Alerts

### Workflow Status

Monitor workflow health:
- Check Actions tab regularly
- Review failed workflows
- Update workflows as needed

### Notifications

Configure notifications in your GitHub settings:
- Settings > Notifications
- Configure for workflow runs
- Set up email or Slack integration

### Discord/Slack Alerts

Workflows send notifications on:
- Build failures
- Security vulnerabilities
- Successful releases
- Migration completions

Configure webhooks:
```bash
gh secret set DISCORD_WEBHOOK
gh secret set SLACK_WEBHOOK
```

## Additional Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [GitHub CLI Manual](https://cli.github.com/manual/)
- [Dependabot Configuration](https://docs.github.com/en/code-security/dependabot)
- [Docker Buildx](https://docs.docker.com/buildx/working-with-buildx/)
- [k6 Load Testing](https://k6.io/docs/)
- [SQLx CLI](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)

## Support

For issues or questions:
- Open an issue in the repository
- Contact the DevOps team
- Review workflow logs
- Check this documentation

---

**Last Updated**: 2025-11-06
**Maintained By**: Tracker Platform Team
