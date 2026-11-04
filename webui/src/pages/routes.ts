import NotFound from './NotFound.svelte';
import Error from './Error.svelte';
import type { Router } from '@/lib';

export { NotFound, Error };

export function register(router: Router) {
	router.register('/', () => import('./Index.svelte'));
	router.register(/^\/sites\/(?<id>[^/]+)$/, () => import('./Site.svelte'));
	router.register('/login', () => import('./Login.svelte'));
	router.register('/users', () => import('./Users.svelte'));
	router.register('/tokens', () => import('./Tokens.svelte'));
	router.register('/me', () => import('./Me.svelte'));
}
