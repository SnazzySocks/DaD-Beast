# Testing Guide

Comprehensive testing documentation for the Tracker Platform.

## Table of Contents

- [Overview](#overview)
- [Running Tests](#running-tests)
- [Test Structure](#test-structure)
- [Writing Tests](#writing-tests)
- [Coverage](#coverage)
- [CI/CD Integration](#cicd-integration)

## Overview

The Tracker Platform uses a comprehensive testing strategy:

1. **Unit Tests** - Test individual functions and modules
2. **Integration Tests** - Test service interactions
3. **E2E Tests** - Test complete user flows
4. **Load Tests** - Test performance under load

### Test Coverage Goals

- **Backend**: 80%+ code coverage
- **Frontend**: 80%+ code coverage
- **Critical paths**: 100% coverage
- **API endpoints**: 100% coverage

## Running Tests

### Quick Start

```bash
# Run all tests
make test-all

# Run specific test suites
make test              # Backend unit + integration
make test-frontend     # Frontend unit tests
make test-e2e          # E2E tests
make test-load         # Load tests
```

### Backend Tests (Rust)

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Specific crate
cargo test -p tracker-auth

# With output
cargo test -- --nocapture

# Watch mode
cargo watch -x test

# Makefile shortcuts
make test-unit
make test-integration
make test-watch
```

### Frontend Tests (Vitest)

```bash
cd frontend

# Run tests
npm test

# Watch mode
npm test -- --watch

# UI mode
npm run test:ui

# Coverage
npm run test:coverage

# Makefile shortcuts
make test-frontend
make test-frontend-ui
make test-frontend-coverage
```

### E2E Tests (Playwright)

```bash
cd frontend

# Run all E2E tests
npm run test:e2e

# Interactive UI mode
npm run test:e2e:ui

# Debug mode
npm run test:e2e:debug

# Headed mode (see browser)
npx playwright test --headed

# Specific browser
npx playwright test --project=chromium

# Specific test file
npx playwright test auth.spec.ts

# Makefile shortcuts
make test-e2e
make test-e2e-ui
```

### Load Tests (k6)

```bash
# Run all load tests
make test-load

# Specific tests
k6 run tests/load/announce.js
k6 run tests/load/api.js
k6 run tests/load/search.js
k6 run tests/load/graphql.js

# Custom parameters
k6 run --vus 100 --duration 5m tests/load/announce.js

# With environment variables
k6 run --env BASE_URL=http://localhost:8080 tests/load/api.js

# Makefile shortcuts
make test-load-announce
make test-load-api
make test-load-search
make test-load-graphql
```

## Test Structure

### Backend (Rust)

```
tracker-platform/
├── crates/
│   ├── auth/
│   │   ├── src/
│   │   │   └── lib.rs          # Unit tests here (#[cfg(test)])
│   │   └── tests/
│   │       └── integration.rs   # Integration tests
│   └── ...
└── tests/
    ├── common/                  # Shared test utilities
    │   ├── mod.rs
    │   ├── fixtures.rs
    │   ├── helpers.rs
    │   └── mocks.rs
    ├── integration/             # Integration tests
    │   ├── test_auth.rs
    │   ├── test_tracker.rs
    │   ├── test_torrent.rs
    │   ├── test_user.rs
    │   ├── test_search.rs
    │   ├── test_api.rs
    │   └── test_community.rs
    └── load/                    # k6 load tests
        ├── announce.js
        ├── api.js
        ├── search.js
        └── graphql.js
```

### Frontend

```
frontend/
├── src/
│   ├── lib/
│   │   ├── components/
│   │   │   └── TorrentCard.svelte
│   │   ├── stores/
│   │   │   └── theme.ts
│   │   └── utils/
│   │       └── formatters.ts
│   └── __tests__/               # Unit tests
│       ├── setup.ts
│       ├── TorrentCard.test.ts
│       ├── ThemeSwitcher.test.ts
│       ├── utils.test.ts
│       └── stores.test.ts
└── tests/
    ├── e2e/                     # E2E tests
    │   ├── auth.spec.ts
    │   ├── torrents.spec.ts
    │   ├── user.spec.ts
    │   ├── forums.spec.ts
    │   ├── chat.spec.ts
    │   └── themes.spec.ts
    ├── fixtures/                # Test data
    └── helpers/                 # Test utilities
```

## Writing Tests

### Backend Unit Tests

```rust
// In src/lib.rs or module files

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";

        // Act
        let result = function_to_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Backend Integration Tests

```rust
// In tests/integration/test_module.rs

use common::{TestContext, fixtures::TestUser};

mod common;

#[tokio::test]
async fn test_integration() {
    let ctx = TestContext::new().await;
    ctx.migrate().await.unwrap();

    // Your test here

    ctx.cleanup().await.unwrap();
}
```

### Frontend Unit Tests

```typescript
// In src/__tests__/Component.test.ts

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Component from '$lib/components/Component.svelte';

describe('Component', () => {
  it('renders correctly', () => {
    render(Component, { props: { value: 'test' } });
    expect(screen.getByText('test')).toBeTruthy();
  });

  it('handles user interaction', async () => {
    const { component } = render(Component);
    await component.handleClick();
    expect(/* assertion */);
  });
});
```

### E2E Tests

```typescript
// In tests/e2e/feature.spec.ts

import { test, expect } from '@playwright/test';

test.describe('Feature', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should do something', async ({ page }) => {
    await page.click('button');
    await expect(page.locator('.result')).toBeVisible();
  });
});
```

### Load Tests

```javascript
// In tests/load/endpoint.js

import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 100 },
    { duration: '1m', target: 100 },
    { duration: '30s', target: 0 },
  ],
};

export default function () {
  const response = http.get('http://localhost:8080/api/endpoint');

  check(response, {
    'status is 200': (r) => r.status === 200,
  });

  sleep(1);
}
```

## Coverage

### Backend Coverage (Rust)

Using `cargo-llvm-cov`:

```bash
# Install
cargo install cargo-llvm-cov

# Generate coverage
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# HTML report
cargo llvm-cov --all-features --workspace --html
open target/llvm-cov/html/index.html

# Makefile shortcut
make coverage
make coverage-open
```

Using `tarpaulin`:

```bash
# Install
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --all-features --workspace --out Html --out Lcov

# Makefile shortcut
make coverage-tarpaulin
```

### Frontend Coverage

```bash
cd frontend

# Run with coverage
npm run test:coverage

# Coverage report is in coverage/ directory
open coverage/index.html

# Makefile shortcut
make test-frontend-coverage
```

### Coverage Thresholds

Coverage thresholds are enforced:

**Backend** (.cargo/config.toml):
- Lines: 80%
- Functions: 80%
- Branches: 75%

**Frontend** (vitest.config.ts):
- Lines: 80%
- Functions: 80%
- Branches: 80%
- Statements: 80%

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test-backend:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:17
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        run: cargo test --all-features

      - name: Generate coverage
        run: |
          cargo install cargo-llvm-cov
          cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./lcov.info

  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: 20

      - name: Install dependencies
        run: cd frontend && npm ci

      - name: Run tests
        run: cd frontend && npm test -- --coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./frontend/coverage/lcov.info

  test-e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: 20

      - name: Install dependencies
        run: cd frontend && npm ci

      - name: Install Playwright
        run: cd frontend && npx playwright install --with-deps

      - name: Run E2E tests
        run: cd frontend && npm run test:e2e

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: frontend/playwright-report/
```

## Test Data & Fixtures

### Using Test Fixtures

```rust
use common::fixtures::{TestUser, TestTorrent, FixtureBuilder};

// Single user
let user = TestUser::new(None, None);

// Custom user
let user = TestUser::new(
    Some("specific_username".to_string()),
    Some("email@example.com".to_string())
);

// Batch creation
let builder = FixtureBuilder::new("test_id".to_string());
let users = builder.users(10);
let torrents = builder.torrents(user_id, 5);
```

### Cleaning Up Test Data

All integration tests should clean up after themselves:

```rust
#[tokio::test]
async fn test_something() {
    let ctx = TestContext::new().await;

    // Test code...

    // Cleanup is automatic via Drop trait
    // Or manual:
    ctx.cleanup().await.unwrap();
}
```

## Best Practices

1. **Isolation**: Each test should be independent
2. **Determinism**: Tests should produce consistent results
3. **Fast**: Keep unit tests fast (<1s each)
4. **Clear**: Use descriptive test names
5. **Arrange-Act-Assert**: Follow AAA pattern
6. **Mocking**: Mock external dependencies
7. **Cleanup**: Always clean up test data
8. **Coverage**: Aim for high coverage, but focus on meaningful tests

## Troubleshooting

### Tests Failing Locally

1. Check database is running:
   ```bash
   docker-compose up -d postgres redis
   ```

2. Reset test database:
   ```bash
   make db-test-reset
   ```

3. Clear build cache:
   ```bash
   cargo clean
   ```

### E2E Tests Failing

1. Update Playwright browsers:
   ```bash
   cd frontend && npx playwright install
   ```

2. Run in headed mode to debug:
   ```bash
   npx playwright test --headed --debug
   ```

3. Check screenshots in `test-results/`

### Coverage Issues

1. Ensure all features are enabled:
   ```bash
   cargo test --all-features
   ```

2. Check coverage tool is installed:
   ```bash
   cargo install cargo-llvm-cov
   ```

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Vitest Documentation](https://vitest.dev/)
- [Playwright Documentation](https://playwright.dev/)
- [k6 Documentation](https://k6.io/docs/)
- [Testing Library](https://testing-library.com/)
