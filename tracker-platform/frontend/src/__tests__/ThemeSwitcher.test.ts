/**
 * Unit tests for ThemeSwitcher component
 */

import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import ThemeSwitcher from '$lib/components/ThemeSwitcher.svelte';
import { theme } from '$lib/stores/theme';

describe('ThemeSwitcher', () => {
  beforeEach(() => {
    // Reset theme before each test
    theme.set('light');
    localStorage.clear();
  });

  it('renders theme switcher button', () => {
    render(ThemeSwitcher);
    expect(screen.getByRole('button')).toBeTruthy();
  });

  it('displays all available themes', async () => {
    render(ThemeSwitcher);

    const button = screen.getByRole('button');
    await fireEvent.click(button);

    // Should show 5 themes: light, dark, cyberpunk, retro, ocean
    expect(screen.getByText(/light/i)).toBeTruthy();
    expect(screen.getByText(/dark/i)).toBeTruthy();
    expect(screen.getByText(/cyberpunk/i)).toBeTruthy();
    expect(screen.getByText(/retro/i)).toBeTruthy();
    expect(screen.getByText(/ocean/i)).toBeTruthy();
  });

  it('switches theme when option is selected', async () => {
    render(ThemeSwitcher);

    const button = screen.getByRole('button');
    await fireEvent.click(button);

    const darkOption = screen.getByText(/dark/i);
    await fireEvent.click(darkOption);

    expect(get(theme)).toBe('dark');
  });

  it('saves theme preference to localStorage', async () => {
    render(ThemeSwitcher);

    const button = screen.getByRole('button');
    await fireEvent.click(button);

    const cyberpunkOption = screen.getByText(/cyberpunk/i);
    await fireEvent.click(cyberpunkOption);

    expect(localStorage.getItem('theme')).toBe('cyberpunk');
  });

  it('applies theme class to document', async () => {
    render(ThemeSwitcher);

    const button = screen.getByRole('button');
    await fireEvent.click(button);

    const retroOption = screen.getByText(/retro/i);
    await fireEvent.click(retroOption);

    expect(document.documentElement.getAttribute('data-theme')).toBe('retro');
  });

  it('loads saved theme on mount', () => {
    localStorage.setItem('theme', 'ocean');

    render(ThemeSwitcher);

    expect(get(theme)).toBe('ocean');
    expect(document.documentElement.getAttribute('data-theme')).toBe('ocean');
  });

  it('defaults to light theme if no preference saved', () => {
    render(ThemeSwitcher);

    expect(get(theme)).toBe('light');
  });

  it('shows current theme as selected', async () => {
    theme.set('dark');

    render(ThemeSwitcher);

    const button = screen.getByRole('button');
    await fireEvent.click(button);

    const darkOption = screen.getByText(/dark/i);
    expect(darkOption.className).toContain('selected');
  });

  it('closes dropdown when theme is selected', async () => {
    const { container } = render(ThemeSwitcher);

    const button = screen.getByRole('button');
    await fireEvent.click(button);

    expect(container.querySelector('.dropdown-open')).toBeTruthy();

    const lightOption = screen.getByText(/light/i);
    await fireEvent.click(lightOption);

    expect(container.querySelector('.dropdown-open')).toBeFalsy();
  });

  it('respects system preference when available', () => {
    // Mock matchMedia to return dark preference
    window.matchMedia = vi.fn().mockImplementation((query) => ({
      matches: query === '(prefers-color-scheme: dark)',
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }));

    render(ThemeSwitcher);

    // If no saved preference, should use system preference
    if (!localStorage.getItem('theme')) {
      expect(get(theme)).toBe('dark');
    }
  });
});
