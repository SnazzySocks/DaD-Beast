/**
 * Unit tests for Svelte stores
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
  theme,
  user,
  notifications,
  searchQuery,
  torrents,
} from '$lib/stores';

describe('Theme Store', () => {
  beforeEach(() => {
    theme.set('light');
  });

  it('has default theme', () => {
    expect(get(theme)).toBe('light');
  });

  it('updates theme', () => {
    theme.set('dark');
    expect(get(theme)).toBe('dark');
  });

  it('validates theme values', () => {
    const validThemes = ['light', 'dark', 'cyberpunk', 'retro', 'ocean'];
    validThemes.forEach((t) => {
      theme.set(t as any);
      expect(get(theme)).toBe(t);
    });
  });
});

describe('User Store', () => {
  beforeEach(() => {
    user.set(null);
  });

  it('starts with null user', () => {
    expect(get(user)).toBeNull();
  });

  it('sets user data', () => {
    const userData = {
      id: '123',
      username: 'testuser',
      email: 'test@example.com',
      ratio: 2.5,
      uploaded: 10000000000,
      downloaded: 4000000000,
    };

    user.set(userData);
    expect(get(user)).toEqual(userData);
  });

  it('clears user on logout', () => {
    user.set({ id: '123', username: 'test' } as any);
    user.set(null);
    expect(get(user)).toBeNull();
  });
});

describe('Notifications Store', () => {
  beforeEach(() => {
    notifications.clear();
  });

  it('starts empty', () => {
    expect(get(notifications)).toHaveLength(0);
  });

  it('adds notification', () => {
    notifications.add({
      id: '1',
      type: 'success',
      message: 'Test notification',
    });

    expect(get(notifications)).toHaveLength(1);
    expect(get(notifications)[0].message).toBe('Test notification');
  });

  it('removes notification', () => {
    notifications.add({ id: '1', type: 'info', message: 'Test' });
    notifications.remove('1');

    expect(get(notifications)).toHaveLength(0);
  });

  it('clears all notifications', () => {
    notifications.add({ id: '1', type: 'info', message: 'Test 1' });
    notifications.add({ id: '2', type: 'info', message: 'Test 2' });

    expect(get(notifications)).toHaveLength(2);

    notifications.clear();

    expect(get(notifications)).toHaveLength(0);
  });

  it('auto-assigns ID if not provided', () => {
    notifications.add({ type: 'info', message: 'Test' } as any);

    const notification = get(notifications)[0];
    expect(notification.id).toBeDefined();
    expect(typeof notification.id).toBe('string');
  });
});

describe('Search Query Store', () => {
  beforeEach(() => {
    searchQuery.set('');
  });

  it('starts with empty query', () => {
    expect(get(searchQuery)).toBe('');
  });

  it('updates search query', () => {
    searchQuery.set('ubuntu');
    expect(get(searchQuery)).toBe('ubuntu');
  });

  it('trims whitespace', () => {
    searchQuery.set('  debian  ');
    expect(get(searchQuery)).toBe('debian');
  });
});

describe('Torrents Store', () => {
  beforeEach(() => {
    torrents.set([]);
  });

  it('starts empty', () => {
    expect(get(torrents)).toHaveLength(0);
  });

  it('sets torrent list', () => {
    const torrentList = [
      { id: '1', name: 'Torrent 1', seeders: 10, leechers: 5 },
      { id: '2', name: 'Torrent 2', seeders: 20, leechers: 10 },
    ];

    torrents.set(torrentList as any);
    expect(get(torrents)).toHaveLength(2);
  });

  it('updates single torrent', () => {
    const torrentList = [
      { id: '1', name: 'Torrent 1', seeders: 10, leechers: 5 },
      { id: '2', name: 'Torrent 2', seeders: 20, leechers: 10 },
    ];

    torrents.set(torrentList as any);

    torrents.update((list) =>
      list.map((t) =>
        t.id === '1' ? { ...t, seeders: 15 } : t
      )
    );

    const updated = get(torrents);
    expect(updated[0].seeders).toBe(15);
    expect(updated[1].seeders).toBe(20);
  });

  it('adds new torrent', () => {
    torrents.set([{ id: '1', name: 'Torrent 1' } as any]);

    torrents.update((list) => [
      ...list,
      { id: '2', name: 'Torrent 2' } as any,
    ]);

    expect(get(torrents)).toHaveLength(2);
  });

  it('removes torrent', () => {
    torrents.set([
      { id: '1', name: 'Torrent 1' },
      { id: '2', name: 'Torrent 2' },
    ] as any);

    torrents.update((list) => list.filter((t) => t.id !== '1'));

    expect(get(torrents)).toHaveLength(1);
    expect(get(torrents)[0].id).toBe('2');
  });
});
