/**
 * K6 Load Test: REST API Endpoints
 * Tests various API endpoints under load
 *
 * Run with:
 *   k6 run --vus 50 --duration 2m api.js
 */

import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { randomIntBetween } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

// Custom metrics
const apiErrors = new Rate('api_errors');
const apiLatency = new Trend('api_latency');
const requestCounter = new Counter('api_requests_total');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 50 },    // Ramp up
    { duration: '1m', target: 100 },    // Increase load
    { duration: '1m', target: 100 },    // Sustain
    { duration: '30s', target: 0 },     // Ramp down
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500', 'p(99)<1000'],
    'http_req_failed': ['rate<0.05'],
    'api_errors': ['rate<0.05'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const API_BASE = `${BASE_URL}/api/v1`;

// Test user credentials (created during setup)
const TEST_USERS = [];
let authToken = null;

export function setup() {
  // In a real scenario, you would create test users and get auth tokens
  // For now, we'll test public endpoints
  return { users: TEST_USERS };
}

export default function (data) {
  // Test different API endpoints with realistic usage patterns

  group('Public API', () => {
    testHealth();
    testListTorrents();
    testSearchTorrents();
    testGetCategories();
  });

  sleep(randomIntBetween(1, 5));
}

function testHealth() {
  const response = http.get(`${BASE_URL}/health`, {
    tags: { name: 'health' },
  });

  requestCounter.add(1);
  const success = check(response, {
    'health status 200': (r) => r.status === 200,
    'health response valid': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.status === 'healthy' || body.status === 'ok';
      } catch {
        return false;
      }
    },
  });

  apiErrors.add(!success);
  apiLatency.add(response.timings.duration);
}

function testListTorrents() {
  const page = randomIntBetween(1, 10);
  const perPage = randomIntBetween(10, 50);

  const response = http.get(`${API_BASE}/torrents?page=${page}&per_page=${perPage}`, {
    tags: { name: 'list_torrents' },
  });

  requestCounter.add(1);
  const success = check(response, {
    'list torrents status 200': (r) => r.status === 200,
    'list torrents has data': (r) => {
      try {
        const body = JSON.parse(r.body);
        return Array.isArray(body.data) || Array.isArray(body.torrents);
      } catch {
        return false;
      }
    },
  });

  apiErrors.add(!success);
  apiLatency.add(response.timings.duration);
}

function testSearchTorrents() {
  const queries = ['ubuntu', 'debian', 'music', 'video', 'software'];
  const query = queries[randomIntBetween(0, queries.length - 1)];

  const response = http.get(`${API_BASE}/search/torrents?q=${query}&limit=20`, {
    tags: { name: 'search_torrents' },
  });

  requestCounter.add(1);
  const success = check(response, {
    'search status 200': (r) => r.status === 200,
    'search has results': (r) => {
      try {
        const body = JSON.parse(r.body);
        return Array.isArray(body.results) || Array.isArray(body.hits);
      } catch {
        return false;
      }
    },
  });

  apiErrors.add(!success);
  apiLatency.add(response.timings.duration);
}

function testGetCategories() {
  const response = http.get(`${API_BASE}/categories`, {
    tags: { name: 'categories' },
  });

  requestCounter.add(1);
  const success = check(response, {
    'categories status 200': (r) => r.status === 200,
    'categories is array': (r) => {
      try {
        const body = JSON.parse(r.body);
        return Array.isArray(body) || Array.isArray(body.data);
      } catch {
        return false;
      }
    },
  });

  apiErrors.add(!success);
  apiLatency.add(response.timings.duration);
}

function testGetTorrentDetails() {
  // This would test getting specific torrent details
  // Requires torrent IDs to be available
  const torrentId = randomIntBetween(1, 1000);

  const response = http.get(`${API_BASE}/torrents/${torrentId}`, {
    tags: { name: 'torrent_details' },
  });

  requestCounter.add(1);
  // Accept 404 as valid (torrent might not exist)
  const success = check(response, {
    'torrent details valid response': (r) => r.status === 200 || r.status === 404,
  });

  if (response.status === 200) {
    apiLatency.add(response.timings.duration);
  }

  apiErrors.add(!success);
}

export function handleSummary(data) {
  const summary = {
    stdout: generateTextSummary(data),
    'load-test-api.json': JSON.stringify(data),
    'load-test-api.html': generateHtmlSummary(data),
  };

  return summary;
}

function generateTextSummary(data) {
  let text = '\n=== API Load Test Summary ===\n\n';

  text += `Total Requests: ${data.metrics.http_reqs.values.count}\n`;
  text += `Requests/sec: ${data.metrics.http_reqs.values.rate.toFixed(2)}\n`;
  text += `Failed Requests: ${(data.metrics.http_req_failed.values.rate * 100).toFixed(2)}%\n\n`;

  text += 'Response Times:\n';
  text += `  Average: ${data.metrics.http_req_duration.values.avg.toFixed(2)}ms\n`;
  text += `  Median:  ${data.metrics.http_req_duration.values.med.toFixed(2)}ms\n`;
  text += `  p(95):   ${data.metrics.http_req_duration.values['p(95)'].toFixed(2)}ms\n`;
  text += `  p(99):   ${data.metrics.http_req_duration.values['p(99)'].toFixed(2)}ms\n`;
  text += `  Max:     ${data.metrics.http_req_duration.values.max.toFixed(2)}ms\n\n`;

  text += 'Data Transfer:\n';
  text += `  Sent:     ${(data.metrics.data_sent.values.count / 1024 / 1024).toFixed(2)} MB\n`;
  text += `  Received: ${(data.metrics.data_received.values.count / 1024 / 1024).toFixed(2)} MB\n\n`;

  return text;
}

function generateHtmlSummary(data) {
  return `
<!DOCTYPE html>
<html>
<head>
  <title>API Load Test Results</title>
  <style>
    body { font-family: Arial, sans-serif; margin: 20px; }
    table { border-collapse: collapse; width: 100%; margin: 20px 0; }
    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
    th { background-color: #4CAF50; color: white; }
    .pass { color: green; }
    .fail { color: red; }
  </style>
</head>
<body>
  <h1>API Load Test Results</h1>
  <h2>Summary</h2>
  <table>
    <tr><th>Metric</th><th>Value</th></tr>
    <tr><td>Total Requests</td><td>${data.metrics.http_reqs.values.count}</td></tr>
    <tr><td>Requests/sec</td><td>${data.metrics.http_reqs.values.rate.toFixed(2)}</td></tr>
    <tr><td>Failed Requests</td><td>${(data.metrics.http_req_failed.values.rate * 100).toFixed(2)}%</td></tr>
    <tr><td>Average Response Time</td><td>${data.metrics.http_req_duration.values.avg.toFixed(2)}ms</td></tr>
    <tr><td>p95 Response Time</td><td>${data.metrics.http_req_duration.values['p(95)'].toFixed(2)}ms</td></tr>
    <tr><td>p99 Response Time</td><td>${data.metrics.http_req_duration.values['p(99)'].toFixed(2)}ms</td></tr>
  </table>
</body>
</html>
`;
}
