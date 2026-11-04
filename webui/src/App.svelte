<script lang="ts">
	import { Writable } from 'chuchi/stores';
	import '@fontsource-variable/bricolage-grotesque';
	import '@fontsource/ibm-plex-mono/400.css';
	import '@fontsource/ibm-plex-mono/600.css';
	import './app.scss';
	import Layout from './layout/Layout.svelte';
	import type { RoutePage } from './main';

	const { page }: { page: Writable<RoutePage> } = $props();

	const Component = $derived($page.component);
	const pageProps = $derived($page.props);
</script>

<Layout>
	{#key $page}
		<Component {...pageProps} />
	{/key}
</Layout>
