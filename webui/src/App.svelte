<script lang="ts">
	import '@fontsource-variable/bricolage-grotesque';
	import '@fontsource/ibm-plex-mono/400.css';
	import '@fontsource/ibm-plex-mono/600.css';
	import './app.scss';
	import Layout from './layout/Layout.svelte';
	import { view } from '@/lib/router.svelte';

	/** The page of the router, see lib/router.svelte.ts. */
	const page = $derived(view.page!);
</script>

<!-- the frame stays from page to page, the page is new -->
<Layout>
	{#key page.key}
		<page.component {...page.props} />
	{/key}
</Layout>
