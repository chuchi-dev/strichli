import { Router, type Request, type Route } from 'chuchi';
import type { Component } from 'svelte';
import { session } from './session.svelte';

/**
 * The pages with chuchi's router, like genco: `pages/routes.ts` registers
 * them, loaded when opened. A page module may export `loadProps` (its data
 * before it shows) and `isPublic` (no login needed).
 */

export type Props = Record<string, any>;

export type PageModule = {
	default: Component<any>;
	// any: each page names the props it takes
	loadProps?: (
		props: any,
		lp: LoadProps,
	) => Promise<Props | void> | Props | void;
	isPublic?: boolean;
};

/** What `loadProps` gets besides the props of the route. */
export class LoadProps {
	redirect: string | null = null;

	constructor(readonly req: Request) {}

	/** Opens another page instead of this one. */
	setRedirect(path: string) {
		this.redirect = path;
	}
}

export const router = new Router<PageModule>();

/** Where the app is. */
class Current {
	path = $state('/');
}

export const current = new Current();

/** Opens a page. */
export function go(path: string, replace = false) {
	router.open(path, {}, { history: replace ? 'replace' : 'push' });
}

/** A page with its props, what App renders. */
export type Page = {
	component: Component<any>;
	props: Props;
	/** a new key renders the page new */
	key: string;
};

export const view = $state<{ page: Page | null }>({ page: null });

type Fallbacks = {
	notFound: () => Promise<{ default: Component<any> }>;
	/** gets `error` */
	error: () => Promise<{ default: Component<any> }>;
	/** the title of a page, a page with its own title sets it itself */
	title: (req: Request) => string;
};

let fallbacks: Fallbacks;

/** The page of a request, its data loaded, or where to go instead. */
async function resolve(
	req: Request,
	route: Route<PageModule> | null,
): Promise<{ page: Page } | { redirect: string }> {
	const key = req.uri;
	if (!route) {
		const NotFound = (await fallbacks.notFound()).default;
		return { page: { component: NotFound, props: {}, key } };
	}

	try {
		const mod = await route.load(req);
		if (!mod.isPublic && !session.user) {
			const back = new URLSearchParams({ url: req.uri });
			return { redirect: `/login?${back}` };
		}
		const lp = new LoadProps(req);
		let props: Props = route.toProps(req);
		props = { ...props, ...((await mod.loadProps?.(props, lp)) ?? {}) };
		if (lp.redirect) return { redirect: lp.redirect };
		return { page: { component: mod.default, props, key } };
	} catch (e) {
		const ErrorPage = (await fallbacks.error()).default;
		return { page: { component: ErrorPage, props: { error: e }, key } };
	}
}

/**
 * Starts the router: `onfirst` is called when the first page is ready, to
 * mount the app.
 */
export function startRouter(pages: Fallbacks, onfirst: () => void) {
	fallbacks = pages;
	router.currentRequest.subscribe(req => {
		if (req) current.path = req.pathname || '/';
	});

	let mounted = false;
	router.onRoute(async (req, route, routing) => {
		const r = await resolve(req, route);
		if ('redirect' in r) {
			router.open(r.redirect, {}, { history: 'replace' });
			await routing.dataReady();
			return;
		}
		if (await routing.dataReady()) return;
		document.title = fallbacks.title(req);
		view.page = r.page;
		if (!mounted) {
			mounted = true;
			onfirst();
		}
		await Promise.resolve();
		await routing.domReady();
	});
	router.initClient();
}
