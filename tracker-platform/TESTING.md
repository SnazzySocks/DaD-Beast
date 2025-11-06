# Testing Quick Reference

> **Complete testing infrastructure for the Tracker Platform**

## 🚀 Quick Start

```bash
# Install all testing tools
make install-tools

# Set up test database
make db-test-setup

# Run all tests
make test-all

# Generate coverage reports
make coverage
make test-frontend-coverage
```

## 📋 Test Commands

### Backend (Rust)

```bash
# Unit tests
cargo test --lib
make test-unit

# Integration tests
cargo test --test '*'
make test-integration

# All tests
cargo test
make test

# Watch mode
cargo watch -x test
make test-watch

# Specific crate
cargo test -p tracker-auth
make test-crate CRATE=tracker-auth
```

### Frontend

```bash
cd frontend

# Unit tests (Vitest)
npm test
npm run test:ui          # Interactive UI
npm run test:coverage    # With coverage

# E2E tests (Playwright)
npm run test:e2e
npm run test:e2e:ui      # Interactive UI
npm run test:e2e:debug   # Debug mode
```

### Load Tests (k6)

```bash
# All load tests
make test-load

# Individual tests
k6 run tests/load/announce.js
k6 run tests/load/api.js
k6 run tests/load/search.js
k6 run tests/load/graphql.js

# Custom parameters
k6 run --vus 1000 --duration 5m tests/load/announce.js
```

## 📊 Coverage

```bash
# Backend coverage (llvm-cov)
make coverage
make coverage-open       # Open HTML report

# Frontend coverage
make test-frontend-coverage
open frontend/coverage/index.html
```

## 🎯 Coverage Targets

- **Backend**: 80%+ (lines, functions, branches)
- **Frontend**: 80%+ (lines, functions, branches, statements)
- **Critical paths**: 100%

## 📁 Test Structure

```
tracker-platform/
├── tests/
│   ├── common/              # Shared utilities
│   ├── integration/         # Integration tests
│   ├── load/               # k6 load tests
│   └── README.md           # Full documentation
├── frontend/
│   ├── src/__tests__/      # Unit tests
│   └── tests/e2e/          # E2E tests
└── .github/workflows/
    └── tests.yml           # CI/CD pipeline
```

## 🔧 CI/CD

Tests run automatically on:
- Every push to `main` or `develop`
- Every pull request
- Load tests run only on `main` branch

## 📚 Documentation

- **Full Guide**: [tests/README.md](/home/user/Projects-1/tracker-platform/tests/README.md)
- **Load Tests**: [tests/load/README.md](/home/user/Projects-1/tracker-platform/tests/load/README.md)
- **Summary**: [tests/SUMMARY.md](/home/user/Projects-1/tracker-platform/tests/SUMMARY.md)

## ✅ Test Checklist

Before committing:
- [ ] All tests pass locally
- [ ] No linting errors
- [ ] Code is formatted
- [ ] Coverage meets thresholds

```bash
# Run full CI locally
make ci
```

## 🐛 Troubleshooting

### Database Issues
```bash
# Reset test database
make db-test-reset
```

### Cache Issues
```bash
# Clean and rebuild
make clean
cargo build
```

### E2E Test Failures
```bash
# Update Playwright browsers
cd frontend && npx playwright install

# Run in headed mode to debug
npx playwright test --headed --debug
```

## 🎓 Writing Tests

### Backend Unit Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = "test";

        // Act
        let result = function(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

### Frontend Unit Test

```typescript
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';

describe('Component', () => {
  it('renders correctly', () => {
    render(Component, { props: { value: 'test' } });
    expect(screen.getByText('test')).toBeTruthy();
  });
});
```

### E2E Test

```typescript
import { test, expect } from '@playwright/test';

test('feature works', async ({ page }) => {
  await page.goto('/');
  await page.click('button');
  await expect(page.locator('.result')).toBeVisible();
});
```

## 🔗 Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Vitest Docs](https://vitest.dev/)
- [Playwright Docs](https://playwright.dev/)
- [k6 Docs](https://k6.io/docs/)
