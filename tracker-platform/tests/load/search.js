/**
 * K6 Load Test: Search Service
 * Tests search endpoint performance and accuracy under load
 *
 * Run with:
 *   k6 run --vus 50 --duration 2m search.js
 */

import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';
import { randomIntBetween, randomItem } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

// Custom metrics
const searchErrors = new Rate('search_errors');
const searchLatency = new Trend('search_latency');
const searchResultCount = new Trend('search_result_count');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 25 },
    { duration: '1m', target: 50 },
    { duration: '1m', target: 75 },
    { duration: '1m', target: 50 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    'http_req_duration': ['p(95)<200', 'p(99)<500'],
    'http_req_failed': ['rate<0.01'],
    'search_latency': ['p(95)<200'],
    'search_errors': ['rate<0.01'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const SEARCH_URL = `${BASE_URL}/api/v1/search`;

// Realistic search queries
const SEARCH_QUERIES = [
  'ubuntu',
  'debian',
  'linux',
  'music',
  'movie',
  'software',
  '1080p',
  'x264',
  'FLAC',
  'documentary',
  'tutorial',
  'open source',
  'game',
  'ebook',
  'series',
  '2024',
  'collection',
  'complete',
  'remaster',
  'original',
];

// Search filters
const CATEGORIES = [1, 2, 3, 4, 5, 6, 7, 8];
const SORT_OPTIONS = ['relevance', 'date', 'size', 'seeders'];

export default function () {
  group('Basic Search', () => {
    testBasicSearch();
  });

  group('Advanced Search', () => {
    testSearchWithFilters();
  });

  group('Search Suggestions', () => {
    testSearchSuggestions();
  });

  sleep(randomIntBetween(1, 3));
}

function testBasicSearch() {
  const query = randomItem(SEARCH_QUERIES);
  const limit = randomIntBetween(10, 50);

  const url = `${SEARCH_URL}/torrents?q=${encodeURIComponent(query)}&limit=${limit}`;

  const startTime = Date.now();
  const response = http.get(url, {
    tags: { name: 'basic_search' },
    timeout: '10s',
  });
  const duration = Date.now() - startTime;

  searchLatency.add(duration);

  const success = check(response, {
    'search status 200': (r) => r.status === 200,
    'search has results': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.results || body.hits || body.data;
      } catch {
        return false;
      }
    },
    'search response time acceptable': (r) => r.timings.duration < 200,
  });

  // Track result count
  try {
    const body = JSON.parse(response.body);
    const results = body.results || body.hits || body.data || [];
    searchResultCount.add(results.length);
  } catch {
    // Ignore parsing errors for metric
  }

  searchErrors.add(!success);
}

function testSearchWithFilters() {
  const query = randomItem(SEARCH_QUERIES);
  const category = randomItem(CATEGORIES);
  const sort = randomItem(SORT_OPTIONS);
  const minSize = randomIntBetween(0, 1000) * 1024 * 1024; // MB to bytes
  const maxSize = minSize + (randomIntBetween(100, 5000) * 1024 * 1024);

  const params = new URLSearchParams({
    q: query,
    category: category,
    sort: sort,
    min_size: minSize,
    max_size: maxSize,
    limit: 20,
  });

  const url = `${SEARCH_URL}/torrents?${params.toString()}`;

  const response = http.get(url, {
    tags: { name: 'filtered_search' },
    timeout: '10s',
  });

  const success = check(response, {
    'filtered search status 200': (r) => r.status === 200,
    'filtered search has structure': (r) => {
      try {
        const body = JSON.parse(r.body);
        return typeof body === 'object';
      } catch {
        return false;
      }
    },
  });

  searchErrors.add(!success);
  searchLatency.add(response.timings.duration);
}

function testSearchSuggestions() {
  const queries = ['ubu', 'deb', 'lin', 'mus', 'mov'];
  const query = randomItem(queries);

  const url = `${SEARCH_URL}/suggest?q=${encodeURIComponent(query)}&limit=10`;

  const response = http.get(url, {
    tags: { name: 'suggestions' },
    timeout: '5s',
  });

  const success = check(response, {
    'suggestions status 200': (r) => r.status === 200,
    'suggestions is array': (r) => {
      try {
        const body = JSON.parse(r.body);
        return Array.isArray(body.suggestions) || Array.isArray(body);
      } catch {
        return false;
      }
    },
    'suggestions fast': (r) => r.timings.duration < 100,
  });

  searchErrors.add(!success);
}

function testUserSearch() {
  const username = 'user' + randomIntBetween(1, 1000);

  const url = `${SEARCH_URL}/users?q=${encodeURIComponent(username)}&limit=10`;

  const response = http.get(url, {
    tags: { name: 'user_search' },
    timeout: '5s',
  });

  const success = check(response, {
    'user search valid': (r) => r.status === 200 || r.status === 404,
  });

  searchErrors.add(!success);
}

export function handleSummary(data) {
  let summary = '\n=== Search Load Test Summary ===\n\n';

  summary += `Total Searches: ${data.metrics.http_reqs.values.count}\n`;
  summary += `Searches/sec: ${data.metrics.http_reqs.values.rate.toFixed(2)}\n`;
  summary += `Failed Searches: ${(data.metrics.http_req_failed.values.rate * 100).toFixed(2)}%\n\n`;

  summary += 'Search Latency:\n';
  summary += `  Average: ${data.metrics.search_latency.values.avg.toFixed(2)}ms\n`;
  summary += `  Median:  ${data.metrics.search_latency.values.med.toFixed(2)}ms\n`;
  summary += `  p(95):   ${data.metrics.search_latency.values['p(95)'].toFixed(2)}ms\n`;
  summary += `  p(99):   ${data.metrics.search_latency.values['p(99)'].toFixed(2)}ms\n\n`;

  if (data.metrics.search_result_count) {
    summary += 'Result Count:\n';
    summary += `  Average: ${data.metrics.search_result_count.values.avg.toFixed(0)} results\n`;
    summary += `  Max:     ${data.metrics.search_result_count.values.max} results\n\n`;
  }

  return {
    'stdout': summary,
    'load-test-search.json': JSON.stringify(data),
  };
}
