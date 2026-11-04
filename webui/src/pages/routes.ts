import type { Request } from 'chuchi';
import { router } from '@/lib/router.svelte';

/** Every page. */
export function registerRoutes() {
	router.register('/', () => import('./Index.svelte'));
	router.register(/^\/sites\/(?<id>[^/]+)$/, () => import('./Site.svelte'));
	router.register('/login', () => import('./Login.svelte'));
	router.register('/password', () => import('./Password.svelte'));
	router.register('/users', () => import('./Users.svelte'));
	router.register('/tokens', () => import('./Tokens.svelte'));
	router.register('/me', () => import('./Me.svelte'));
}

const TITLES: Record<string, string> = {
	'/login': 'Login',
	'/password': 'Set the password',
	'/users': 'Users',
	'/tokens': 'API tokens',
	'/me': 'Account',
};

/** pages with their own title (a site) set it themselves */
export function pageTitle(req: Request): string {
	const title = TITLES[req.pathname];
	return title ? `${title} - strichli` : 'strichli';
}
