import { writable } from 'svelte/store';
import { browser } from '$app/environment';

type Theme = 'light' | 'dark';

// Get initial theme from localStorage or system preference
const getInitialTheme = (): Theme => {
	if (!browser) return 'light';

	const savedTheme = localStorage.getItem('theme') as Theme;
	if (savedTheme) return savedTheme;

	return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
};

// Create the theme store
const theme = writable<Theme>(getInitialTheme());

// Subscribe to changes and update localStorage and data-theme attribute
if (browser) {
	theme.subscribe((value) => {
		localStorage.setItem('theme', value);
		document.documentElement.setAttribute('data-theme', value);
	});
}

// Function to toggle theme
const toggleTheme = () => {
	theme.update((current) => (current === 'light' ? 'dark' : 'light'));
};

export { theme, toggleTheme };
