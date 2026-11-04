/** light, dark or the system's, per browser */
export type Theme = 'system' | 'light' | 'dark';

const STORAGE_KEY = 'STRICHLI_THEME';

export function getTheme(): Theme {
	const t = localStorage.getItem(STORAGE_KEY);
	return t === 'light' || t === 'dark' ? t : 'system';
}

/** Sets the theme on the page (app.scss reads `data-theme`). */
export function applyTheme(theme: Theme = getTheme()) {
	if (theme === 'system') delete document.documentElement.dataset.theme;
	else document.documentElement.dataset.theme = theme;
}

export function setTheme(theme: Theme) {
	if (theme === 'system') localStorage.removeItem(STORAGE_KEY);
	else localStorage.setItem(STORAGE_KEY, theme);
	applyTheme(theme);
}
