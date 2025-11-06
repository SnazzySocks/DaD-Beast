/**
 * Unit tests for TorrentCard component
 */

import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import TorrentCard from '$lib/components/TorrentCard.svelte';

describe('TorrentCard', () => {
  const mockTorrent = {
    id: '123',
    name: 'Ubuntu 22.04 LTS Desktop',
    size: 3_500_000_000, // 3.5 GB
    seeders: 42,
    leechers: 15,
    downloads: 1250,
    category: {
      name: 'Software',
      icon: '💿',
    },
    uploader: {
      username: 'testuser',
      ratio: 2.5,
    },
    createdAt: '2024-01-15T10:00:00Z',
  };

  it('renders torrent name', () => {
    render(TorrentCard, { props: { torrent: mockTorrent } });
    expect(screen.getByText('Ubuntu 22.04 LTS Desktop')).toBeTruthy();
  });

  it('displays formatted file size', () => {
    render(TorrentCard, { props: { torrent: mockTorrent } });
    // Should show "3.5 GB" or similar
    expect(screen.getByText(/3\.5\s*GB/i)).toBeTruthy();
  });

  it('shows seeder and leecher counts', () => {
    render(TorrentCard, { props: { torrent: mockTorrent } });
    expect(screen.getByText(/42/)).toBeTruthy(); // seeders
    expect(screen.getByText(/15/)).toBeTruthy(); // leechers
  });

  it('displays category', () => {
    render(TorrentCard, { props: { torrent: mockTorrent } });
    expect(screen.getByText('Software')).toBeTruthy();
  });

  it('shows uploader username', () => {
    render(TorrentCard, { props: { torrent: mockTorrent } });
    expect(screen.getByText('testuser')).toBeTruthy();
  });

  it('applies correct CSS classes based on seeder count', () => {
    const { container } = render(TorrentCard, { props: { torrent: mockTorrent } });

    // High seeders (>10) should have green indicator
    const seederElement = container.querySelector('.seeders');
    expect(seederElement?.className).toContain('healthy');
  });

  it('shows warning for low seeder count', () => {
    const lowSeederTorrent = { ...mockTorrent, seeders: 2 };
    const { container } = render(TorrentCard, {
      props: { torrent: lowSeederTorrent },
    });

    const seederElement = container.querySelector('.seeders');
    expect(seederElement?.className).toContain('warning');
  });

  it('shows dead torrent indicator when no seeders', () => {
    const deadTorrent = { ...mockTorrent, seeders: 0 };
    const { container } = render(TorrentCard, { props: { torrent: deadTorrent } });

    const seederElement = container.querySelector('.seeders');
    expect(seederElement?.className).toContain('dead');
  });

  it('formats download count correctly', () => {
    render(TorrentCard, { props: { torrent: mockTorrent } });
    expect(screen.getByText(/1\.2k/i)).toBeTruthy(); // 1250 -> 1.2k
  });

  it('handles missing optional fields gracefully', () => {
    const minimalTorrent = {
      id: '456',
      name: 'Test Torrent',
      size: 1000000,
      seeders: 1,
      leechers: 0,
      downloads: 0,
    };

    expect(() => {
      render(TorrentCard, { props: { torrent: minimalTorrent } });
    }).not.toThrow();
  });

  it('emits click event when clicked', async () => {
    const { component } = render(TorrentCard, { props: { torrent: mockTorrent } });

    const clicked = vi.fn();
    component.$on('click', clicked);

    const card = screen.getByRole('article');
    await card.click();

    expect(clicked).toHaveBeenCalledTimes(1);
  });
});
