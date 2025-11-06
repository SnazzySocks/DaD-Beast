# Load Testing with K6

This directory contains k6 load testing scripts for the tracker platform.

## Prerequisites

Install k6:

```bash
# macOS
brew install k6

# Ubuntu/Debian
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update
sudo apt-get install k6

# Docker
docker pull grafana/k6
```

## Running Tests

### Basic Usage

```bash
# Run with default settings
k6 run announce.js

# Run with custom VUs and duration
k6 run --vus 100 --duration 30s announce.js

# Run with environment variable
k6 run --env BASE_URL=http://localhost:8080 announce.js
```

### Test Scripts

#### 1. Announce Test (`announce.js`)
Tests BitTorrent tracker announce endpoint performance.

**Target:** 10,000 announces/second

```bash
# Light test
k6 run --vus 100 --duration 1m announce.js

# Stress test
k6 run --vus 1000 --duration 5m announce.js

# Spike test
k6 run announce.js  # Uses defined stages
```

**Metrics:**
- Announce latency (should be <100ms p95)
- Error rate (should be <1%)
- Throughput (requests/sec)

#### 2. API Test (`api.js`)
Tests REST API endpoints under realistic load.

```bash
# Standard load
k6 run --vus 50 --duration 2m api.js

# Heavy load
k6 run --vus 200 --duration 5m api.js
```

**Tests:**
- Health check
- List torrents
- Search torrents
- Get categories
- Torrent details

#### 3. Search Test (`search.js`)
Tests search service performance with various queries and filters.

```bash
# Basic search test
k6 run --vus 50 --duration 2m search.js

# Search stress test
k6 run --vus 100 --duration 5m search.js
```

**Tests:**
- Basic text search
- Search with filters
- Search suggestions
- User search

#### 4. GraphQL Test (`graphql.js`)
Tests GraphQL endpoint with complex queries.

```bash
# Standard GraphQL load
k6 run --vus 50 --duration 2m graphql.js

# Complex query test
k6 run --vus 100 --duration 3m graphql.js
```

**Tests:**
- List queries
- Detail queries with nested data
- Search queries
- DataLoader efficiency

## Test Stages

All tests use staged load profiles:

1. **Ramp-up**: Gradually increase load
2. **Sustain**: Maintain peak load
3. **Stress** (optional): Push beyond normal capacity
4. **Ramp-down**: Gracefully decrease load

Example stage configuration:

```javascript
stages: [
  { duration: '30s', target: 100 },   // Ramp up
  { duration: '2m', target: 500 },    // Peak load
  { duration: '1m', target: 1000 },   // Stress
  { duration: '30s', target: 0 },     // Ramp down
]
```

## Thresholds

Tests define performance thresholds:

```javascript
thresholds: {
  'http_req_duration': ['p(95)<100', 'p(99)<200'],  // Response times
  'http_req_failed': ['rate<0.01'],                 // Error rate <1%
}
```

## Results and Reporting

### Console Output

Real-time metrics are displayed during test execution:

```
scenarios: (100.00%) 1 scenario, 100 max VUs, 2m30s max duration
✓ status is 200
✓ response has interval

checks.........................: 100.00% ✓ 50000  ✗ 0
data_received..................: 125 MB  1.0 MB/s
data_sent......................: 25 MB   208 kB/s
http_req_duration..............: avg=45ms min=10ms med=42ms max=250ms p(95)=85ms p(99)=150ms
http_reqs......................: 50000   416/s
```

### JSON Output

Results are saved as JSON for further analysis:

```bash
k6 run --out json=results.json announce.js
```

### HTML Reports

Generate HTML reports using k6-reporter:

```bash
k6 run --out json=results.json announce.js
k6-reporter results.json --output report.html
```

### Cloud Results

Send results to k6 Cloud:

```bash
k6 run --out cloud announce.js
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Load Tests

on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM
  workflow_dispatch:

jobs:
  load-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup infrastructure
        run: docker-compose up -d

      - name: Run k6 tests
        uses: grafana/k6-action@v0.3.0
        with:
          filename: tests/load/announce.js
          flags: --vus 100 --duration 2m

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: k6-results
          path: load-test-*.json
```

## Performance Targets

### Tracker Announces
- **Target:** 10,000 requests/second
- **p95 latency:** <100ms
- **p99 latency:** <200ms
- **Error rate:** <0.1%

### API Endpoints
- **Target:** 1,000 requests/second
- **p95 latency:** <500ms
- **p99 latency:** <1000ms
- **Error rate:** <1%

### Search
- **Target:** 500 queries/second
- **p95 latency:** <200ms
- **p99 latency:** <500ms
- **Result accuracy:** >95%

### GraphQL
- **Target:** 500 requests/second
- **p95 latency:** <500ms
- **Complex queries:** <1000ms
- **N+1 queries:** Prevented by DataLoader

## Troubleshooting

### High Error Rates

1. Check application logs
2. Verify database connections
3. Check rate limiting configuration
4. Monitor resource usage (CPU, memory)

### High Latency

1. Check database query performance
2. Verify cache hit rates
3. Check network latency
4. Monitor database connection pool

### Resource Exhaustion

1. Increase database connection pool
2. Scale horizontally
3. Optimize queries
4. Enable caching

## Best Practices

1. **Start Small**: Begin with low VU count and short duration
2. **Ramp Up**: Use staged load to simulate realistic traffic patterns
3. **Set Baselines**: Establish performance baselines before optimization
4. **Monitor**: Watch system metrics during tests
5. **Iterate**: Run tests multiple times for consistency
6. **Production-like**: Test against environment similar to production
7. **Regular Testing**: Run load tests regularly, not just before release

## Custom Scenarios

Create custom test scenarios:

```javascript
export const options = {
  scenarios: {
    announce_heavy: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 1000 },
        { duration: '5m', target: 1000 },
      ],
      exec: 'announceTest',
    },
    api_moderate: {
      executor: 'constant-vus',
      vus: 50,
      duration: '5m',
      exec: 'apiTest',
    },
  },
};
```

## Resources

- [k6 Documentation](https://k6.io/docs/)
- [k6 Examples](https://k6.io/docs/examples/)
- [Performance Testing Guide](https://k6.io/docs/testing-guides/)
