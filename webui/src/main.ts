import { mount } from 'svelte';
import App from './App.svelte';
import { startRouter } from './lib/router.svelte';
import { session } from './lib/session.svelte';
import { applyTheme } from './lib/theme';
import { pageTitle, registerRoutes } from './pages/routes';

applyTheme();
registerRoutes();
// who is logged in, before the first page
session.init().then(() =>
	startRouter(
		{
			notFound: () => import('./pages/NotFound.svelte'),
			error: () => import('./pages/Error.svelte'),
			title: pageTitle,
		},
		() => mount(App, { target: document.body }),
	),
);
