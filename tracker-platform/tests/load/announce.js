/**
 * K6 Load Test: BitTorrent Tracker Announces
 * Target: 10,000 announces/second
 *
 * Run with:
 *   k6 run --vus 100 --duration 30s announce.js
 *   k6 run --vus 500 --duration 5m announce.js  # stress test
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';
import { randomString, randomIntBetween } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

// Custom metrics
const announceErrors = new Rate('announce_errors');
const announceLatency = new Trend('announce_latency');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 100 },   // Ramp up to 100 users
    { duration: '1m', target: 500 },    // Ramp up to 500 users
    { duration: '2m', target: 1000 },   // Ramp up to 1000 users
    { duration: '2m', target: 1000 },   // Stay at 1000 users
    { duration: '30s', target: 0 },     // Ramp down
  ],
  thresholds: {
    'http_req_duration': ['p(95)<100', 'p(99)<200'], // 95% < 100ms, 99% < 200ms
    'http_req_failed': ['rate<0.01'],                // Error rate < 1%
    'announce_errors': ['rate<0.01'],
  },
};

// Test data
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const INFO_HASHES = generateInfoHashes(100); // 100 different torrents

function generateInfoHashes(count) {
  const hashes = [];
  for (let i = 0; i < count; i++) {
    hashes.push(randomString(40, '0123456789abcdef'));
  }
  return hashes;
}

function generatePeerId() {
  return '-TR3000-' + randomString(12, '0123456789');
}

function urlEncode(str) {
  return encodeURIComponent(str);
}

export default function () {
  // Select random info hash
  const infoHash = INFO_HASHES[randomIntBetween(0, INFO_HASHES.length - 1)];
  const peerId = generatePeerId();

  // Random peer stats
  const port = randomIntBetween(6881, 6999);
  const uploaded = randomIntBetween(0, 10000000000);
  const downloaded = randomIntBetween(0, 5000000000);
  const left = randomIntBetween(0, 1000000000);
  const event = left === 0 ? 'completed' : 'started';

  // Build announce URL
  const params = [
    `info_hash=${urlEncode(infoHash)}`,
    `peer_id=${urlEncode(peerId)}`,
    `port=${port}`,
    `uploaded=${uploaded}`,
    `downloaded=${downloaded}`,
    `left=${left}`,
    `event=${event}`,
    `compact=1`,
    `numwant=50`,
  ].join('&');

  const url = `${BASE_URL}/tracker/announce?${params}`;

  // Make announce request
  const startTime = Date.now();
  const response = http.get(url, {
    tags: { name: 'announce' },
    timeout: '10s',
  });
  const duration = Date.now() - startTime;

  // Record metrics
  announceLatency.add(duration);

  // Check response
  const success = check(response, {
    'status is 200': (r) => r.status === 200,
    'response has interval': (r) => r.body && r.body.includes('interval'),
    'response time < 100ms': (r) => r.timings.duration < 100,
  });

  announceErrors.add(!success);

  // Simulate realistic announce interval (30-60 seconds)
  sleep(randomIntBetween(1, 3) / 10); // 0.1-0.3s for load testing
}

export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'load-test-announce.json': JSON.stringify(data),
  };
}

function textSummary(data, options) {
  const indent = options.indent || '';
  const colors = options.enableColors || false;

  let summary = `\n${indent}Announce Load Test Summary:\n`;
  summary += `${indent}=====================================\n`;
  summary += `${indent}Total Requests: ${data.metrics.http_reqs.values.count}\n`;
  summary += `${indent}Failed Requests: ${data.metrics.http_req_failed.values.rate * 100}%\n`;
  summary += `${indent}Request Duration (avg): ${data.metrics.http_req_duration.values.avg.toFixed(2)}ms\n`;
  summary += `${indent}Request Duration (p95): ${data.metrics.http_req_duration.values['p(95)'].toFixed(2)}ms\n`;
  summary += `${indent}Request Duration (p99): ${data.metrics.http_req_duration.values['p(99)'].toFixed(2)}ms\n`;
  summary += `${indent}Requests/sec: ${data.metrics.http_reqs.values.rate.toFixed(2)}\n`;

  return summary;
}
