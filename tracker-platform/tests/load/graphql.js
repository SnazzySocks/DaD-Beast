/**
 * K6 Load Test: GraphQL Endpoint
 * Tests GraphQL queries and mutations under load
 *
 * Run with:
 *   k6 run --vus 50 --duration 2m graphql.js
 */

import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';
import { randomIntBetween, randomItem } from 'https://jslib.k6.io/k6-utils/1.2.0/index.js';

// Custom metrics
const graphqlErrors = new Rate('graphql_errors');
const graphqlLatency = new Trend('graphql_latency');
const graphqlDataSize = new Trend('graphql_data_size');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 25 },
    { duration: '1m', target: 50 },
    { duration: '1m', target: 75 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500', 'p(99)<1000'],
    'http_req_failed': ['rate<0.01'],
    'graphql_errors': ['rate<0.01'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const GRAPHQL_URL = `${BASE_URL}/graphql`;

// GraphQL queries
const QUERIES = {
  torrents: `
    query GetTorrents($first: Int!, $after: String) {
      torrents(first: $first, after: $after) {
        edges {
          node {
            id
            name
            size
            seeders
            leechers
            downloads
            uploader {
              username
              ratio
            }
            category {
              name
            }
          }
          cursor
        }
        pageInfo {
          hasNextPage
          endCursor
        }
        totalCount
      }
    }
  `,

  torrentDetail: `
    query GetTorrent($id: ID!) {
      torrent(id: $id) {
        id
        name
        description
        size
        infoHash
        seeders
        leechers
        downloads
        createdAt
        uploader {
          id
          username
          uploaded
          downloaded
          ratio
        }
        files {
          path
          size
        }
        comments(first: 10) {
          edges {
            node {
              id
              content
              user {
                username
              }
              createdAt
            }
          }
        }
      }
    }
  `,

  searchTorrents: `
    query SearchTorrents($query: String!, $first: Int!) {
      searchTorrents(query: $query, first: $first) {
        edges {
          node {
            id
            name
            size
            seeders
            category {
              name
            }
          }
        }
        totalCount
      }
    }
  `,

  userProfile: `
    query GetUser($id: ID!) {
      user(id: $id) {
        id
        username
        uploaded
        downloaded
        ratio
        bonusPoints
        joinedAt
        torrents(first: 5) {
          edges {
            node {
              id
              name
              seeders
            }
          }
        }
      }
    }
  `,
};

export default function () {
  group('GraphQL Queries', () => {
    testListTorrents();
    testTorrentDetail();
    testSearchTorrents();
  });

  sleep(randomIntBetween(1, 3));
}

function testListTorrents() {
  const variables = {
    first: randomIntBetween(10, 50),
    after: null,
  };

  const payload = JSON.stringify({
    query: QUERIES.torrents,
    variables: variables,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { name: 'graphql_list_torrents' },
  };

  const response = http.post(GRAPHQL_URL, payload, params);

  const success = check(response, {
    'graphql status 200': (r) => r.status === 200,
    'no graphql errors': (r) => {
      try {
        const body = JSON.parse(r.body);
        return !body.errors;
      } catch {
        return false;
      }
    },
    'has data': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.data && body.data.torrents;
      } catch {
        return false;
      }
    },
  });

  graphqlErrors.add(!success);
  graphqlLatency.add(response.timings.duration);
  graphqlDataSize.add(response.body.length);
}

function testTorrentDetail() {
  const torrentId = randomIntBetween(1, 1000);

  const variables = {
    id: torrentId.toString(),
  };

  const payload = JSON.stringify({
    query: QUERIES.torrentDetail,
    variables: variables,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { name: 'graphql_torrent_detail' },
  };

  const response = http.post(GRAPHQL_URL, payload, params);

  const success = check(response, {
    'graphql status 200': (r) => r.status === 200,
    'no errors or not found': (r) => {
      try {
        const body = JSON.parse(r.body);
        // Accept null data (torrent not found) as success
        return !body.errors || body.data.torrent === null;
      } catch {
        return false;
      }
    },
  });

  graphqlErrors.add(!success);
  graphqlLatency.add(response.timings.duration);
}

function testSearchTorrents() {
  const queries = ['ubuntu', 'debian', 'music', 'video', 'software'];
  const searchQuery = randomItem(queries);

  const variables = {
    query: searchQuery,
    first: 20,
  };

  const payload = JSON.stringify({
    query: QUERIES.searchTorrents,
    variables: variables,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { name: 'graphql_search' },
  };

  const response = http.post(GRAPHQL_URL, payload, params);

  const success = check(response, {
    'search status 200': (r) => r.status === 200,
    'search has structure': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.data && typeof body.data === 'object';
      } catch {
        return false;
      }
    },
  });

  graphqlErrors.add(!success);
  graphqlLatency.add(response.timings.duration);
}

function testComplexQuery() {
  // Test query complexity and DataLoader efficiency
  const query = `
    query ComplexQuery {
      torrents(first: 20) {
        edges {
          node {
            id
            name
            uploader {
              username
              torrents(first: 5) {
                edges {
                  node {
                    name
                  }
                }
              }
            }
            comments(first: 5) {
              edges {
                node {
                  content
                  user {
                    username
                  }
                }
              }
            }
          }
        }
      }
    }
  `;

  const payload = JSON.stringify({ query });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
    tags: { name: 'graphql_complex' },
  };

  const response = http.post(GRAPHQL_URL, payload, params);

  check(response, {
    'complex query succeeds': (r) => r.status === 200,
    'complex query reasonably fast': (r) => r.timings.duration < 1000,
  });

  graphqlLatency.add(response.timings.duration);
}

export function handleSummary(data) {
  let summary = '\n=== GraphQL Load Test Summary ===\n\n';

  summary += `Total Requests: ${data.metrics.http_reqs.values.count}\n`;
  summary += `Requests/sec: ${data.metrics.http_reqs.values.rate.toFixed(2)}\n`;
  summary += `Failed Requests: ${(data.metrics.http_req_failed.values.rate * 100).toFixed(2)}%\n\n`;

  summary += 'Response Times:\n';
  summary += `  Average: ${data.metrics.graphql_latency.values.avg.toFixed(2)}ms\n`;
  summary += `  p(95):   ${data.metrics.graphql_latency.values['p(95)'].toFixed(2)}ms\n`;
  summary += `  p(99):   ${data.metrics.graphql_latency.values['p(99)'].toFixed(2)}ms\n\n`;

  if (data.metrics.graphql_data_size) {
    summary += 'Response Size:\n';
    summary += `  Average: ${(data.metrics.graphql_data_size.values.avg / 1024).toFixed(2)} KB\n`;
    summary += `  Max:     ${(data.metrics.graphql_data_size.values.max / 1024).toFixed(2)} KB\n\n`;
  }

  return {
    'stdout': summary,
    'load-test-graphql.json': JSON.stringify(data),
  };
}
