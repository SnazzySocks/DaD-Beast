/**
 * Unit tests for utility functions
 */

import { describe, it, expect } from 'vitest';
import {
  formatBytes,
  formatNumber,
  formatDate,
  formatDuration,
  calculateRatio,
  truncateString,
  debounce,
  throttle,
} from '$lib/utils';

describe('formatBytes', () => {
  it('formats bytes correctly', () => {
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(1024)).toBe('1.0 KB');
    expect(formatBytes(1024 * 1024)).toBe('1.0 MB');
    expect(formatBytes(1024 * 1024 * 1024)).toBe('1.0 GB');
    expect(formatBytes(1024 * 1024 * 1024 * 1024)).toBe('1.0 TB');
  });

  it('handles decimal places', () => {
    expect(formatBytes(1536, 2)).toBe('1.50 KB'); // 1.5 KB
    expect(formatBytes(3_500_000_000, 1)).toBe('3.3 GB');
  });

  it('handles negative numbers', () => {
    expect(formatBytes(-1024)).toBe('-1.0 KB');
  });
});

describe('formatNumber', () => {
  it('formats numbers with k suffix', () => {
    expect(formatNumber(1250)).toBe('1.2k');
    expect(formatNumber(5400)).toBe('5.4k');
    expect(formatNumber(999)).toBe('999');
  });

  it('formats numbers with M suffix', () => {
    expect(formatNumber(1_250_000)).toBe('1.2M');
    expect(formatNumber(3_500_000)).toBe('3.5M');
  });

  it('handles small numbers', () => {
    expect(formatNumber(0)).toBe('0');
    expect(formatNumber(42)).toBe('42');
    expect(formatNumber(999)).toBe('999');
  });
});

describe('formatDate', () => {
  it('formats ISO date strings', () => {
    const date = '2024-01-15T10:30:00Z';
    const formatted = formatDate(date);
    expect(formatted).toMatch(/Jan 15, 2024/);
  });

  it('handles relative dates', () => {
    const now = new Date();
    const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000);
    const formatted = formatDate(yesterday.toISOString(), { relative: true });
    expect(formatted).toMatch(/yesterday|1 day ago/i);
  });

  it('handles custom formats', () => {
    const date = '2024-01-15T10:30:00Z';
    const formatted = formatDate(date, { format: 'short' });
    expect(formatted).toMatch(/1\/15\/24/);
  });
});

describe('formatDuration', () => {
  it('formats seconds correctly', () => {
    expect(formatDuration(45)).toBe('45s');
    expect(formatDuration(90)).toBe('1m 30s');
    expect(formatDuration(3665)).toBe('1h 1m 5s');
  });

  it('handles zero duration', () => {
    expect(formatDuration(0)).toBe('0s');
  });

  it('handles days', () => {
    expect(formatDuration(86400)).toBe('1d 0h 0m 0s');
    expect(formatDuration(90000)).toBe('1d 1h 0m 0s');
  });
});

describe('calculateRatio', () => {
  it('calculates ratio correctly', () => {
    expect(calculateRatio(10000, 5000)).toBe(2.0);
    expect(calculateRatio(7500, 2500)).toBe(3.0);
    expect(calculateRatio(1000, 1000)).toBe(1.0);
  });

  it('handles zero downloaded', () => {
    expect(calculateRatio(10000, 0)).toBe(Infinity);
  });

  it('handles zero uploaded', () => {
    expect(calculateRatio(0, 5000)).toBe(0.0);
  });

  it('handles both zero', () => {
    expect(calculateRatio(0, 0)).toBe(0.0);
  });
});

describe('truncateString', () => {
  it('truncates long strings', () => {
    const longString = 'This is a very long string that should be truncated';
    expect(truncateString(longString, 20)).toBe('This is a very long...');
  });

  it('does not truncate short strings', () => {
    const shortString = 'Short';
    expect(truncateString(shortString, 20)).toBe('Short');
  });

  it('handles custom ellipsis', () => {
    const string = 'This is a test string';
    expect(truncateString(string, 10, '…')).toBe('This is a…');
  });

  it('handles exact length', () => {
    const string = 'Exact';
    expect(truncateString(string, 5)).toBe('Exact');
  });
});

describe('debounce', () => {
  it('debounces function calls', async () => {
    let callCount = 0;
    const fn = () => callCount++;
    const debounced = debounce(fn, 100);

    debounced();
    debounced();
    debounced();

    expect(callCount).toBe(0);

    await new Promise((resolve) => setTimeout(resolve, 150));

    expect(callCount).toBe(1);
  });

  it('passes arguments to debounced function', async () => {
    let result = 0;
    const fn = (x: number) => {
      result = x;
    };
    const debounced = debounce(fn, 100);

    debounced(42);

    await new Promise((resolve) => setTimeout(resolve, 150));

    expect(result).toBe(42);
  });
});

describe('throttle', () => {
  it('throttles function calls', async () => {
    let callCount = 0;
    const fn = () => callCount++;
    const throttled = throttle(fn, 100);

    throttled(); // Called immediately
    throttled(); // Ignored
    throttled(); // Ignored

    expect(callCount).toBe(1);

    await new Promise((resolve) => setTimeout(resolve, 150));

    throttled(); // Called after delay

    expect(callCount).toBe(2);
  });

  it('preserves function context', async () => {
    const obj = {
      value: 42,
      getValue: function () {
        return this.value;
      },
    };

    const throttled = throttle(obj.getValue.bind(obj), 100);

    expect(throttled()).toBe(42);
  });
});
