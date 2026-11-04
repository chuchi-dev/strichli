import type { Request } from 'chuchi';
import { getContext } from 'svelte';
import * as routes from './pages/routes';
import type LoadProps from './lib/LoadProps';
import type Session from './lib/Session';
import type { Component, ComponentModule, Route, Router } from './lib';

export function getRouter(): Router {
	return getContext('router');
}

export function getSession(): Session {
	return getContext('session');
}

export type RoutePage = {
	component: Component;
	props: Record<string, any>;
};

export type RouteResponse = {
	page?: RoutePage;
	redirect?: string;
};

const TITLES: Record<string, string> = {
	'/login': 'Login',
	'/password': 'Set the password',
	'/users': 'Users',
	'/tokens': 'API tokens',
	'/me': 'Account',
};

/** pages with their own title (a site) set it themselves */
export function pageTitle(req: Request): string {
	const title = TITLES[req.url.pathname];
	return title ? `${title} - strichli` : 'strichli';
}

export async function handleRoute(
	req: Request,
	route: Route | null,
	loadProps: LoadProps,
): Promise<RouteResponse> {
	if (!route) {
		return { page: { component: routes.NotFound, props: {} } };
	}

	let comp: ComponentModule;
	let pageProps: Record<string, any>;
	try {
		comp = await route.load(req);

		if (!comp.isPublic && !loadProps.session.isLoggedIn()) {
			return { redirect: '/login' };
		}

		pageProps = route.toProps(req);

		if (typeof comp.loadProps === 'function') {
			const nProps = await comp.loadProps(pageProps, loadProps);
			if (nProps) pageProps = { ...pageProps, ...nProps };
		}

		if (loadProps.redirect) return { redirect: loadProps.redirect };
	} catch (e) {
		console.log('error', e);
		return {
			page: { component: routes.Error, props: { error: e } },
		};
	}

	return { page: { component: comp.default, props: pageProps } };
}
