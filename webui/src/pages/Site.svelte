<script module lang="ts">
	import { getSite, getStats } from '@/api/sites';
	import { rangeFromProps } from '@/lib/range';

	type Props = { id: string; range?: string; from?: string; to?: string };

	export async function loadProps(props: Props) {
		const { range, preset } = rangeFromProps(props);
		const [site, stats] = await Promise.all([
			getSite(props.id),
			getStats(props.id, range),
		]);

		return { site, stats, current: range, preset };
	}
</script>

<script lang="ts">
	import type { Range, Site, Stats, ValueCount } from '@/api/sites';
	import { deleteSite, getValues, updateSite } from '@/api/sites';
	import BarList from '@/components/BarList.svelte';
	import Button from '@/components/Button.svelte';
	import Chart from '@/components/Chart.svelte';
	import Copy from '@/components/Copy.svelte';
	import Field from '@/components/Field.svelte';
	import RangePicker from '@/components/RangePicker.svelte';
	import { num } from '@/lib/format';
	import { getRouter, getSession } from '@/main';

	type Loaded = {
		site: Site;
		stats: Stats;
		current: Range;
		preset: string | null;
		/** just added, show how to include the script */
		new?: string;
	};

	const { site, stats, current, preset, new: isNew }: Loaded = $props();

	const router = getRouter();
	const user = getSession().user;

	$effect(() => {
		document.title = `${site.name} - strichli`;
	});

	const base = $derived(`/sites/${site.id}`);

	// keys
	let key: string | null = $state(null);
	let values: ValueCount[] = $state([]);
	let valuesError = $state('');

	async function selectKey(k: string) {
		if (key === k) {
			key = null;
			return;
		}

		key = k;
		values = [];
		valuesError = '';
		try {
			values = await getValues(site.id, k, current);
		} catch (e: any) {
			valuesError = e.message;
		}
	}

	const fields = $derived(
		Object.entries(
			values.reduce<Record<string, ValueCount[]>>((acc, v) => {
				(acc[v.field] ??= []).push(v);
				return acc;
			}, {}),
		),
	);

	// settings
	// svelte-ignore state_referenced_locally
	let showSettings = $state(!!isNew);
	// svelte-ignore state_referenced_locally
	let domain = $state(site.domain);
	// svelte-ignore state_referenced_locally
	let name = $state(site.name);
	let settingsError = $state('');

	async function save(e: SubmitEvent) {
		e.preventDefault();
		settingsError = '';

		try {
			await updateSite(site.id, domain, name);
			router.open(base);
		} catch (e: any) {
			settingsError = e.message;
		}
	}

	async function remove() {
		const sure = confirm(
			`Delete ${site.domain} and all its visits? This can't be undone.`,
		);
		if (!sure) return;

		try {
			await deleteSite(site.id);
			router.open('/');
		} catch (e: any) {
			settingsError = e.message;
		}
	}

	const keyExample = `strichli('signup', { plan: 'pro' });`;
</script>

<div class="title">
	<div>
		<h1>{site.name}</h1>
		<a
			class="domain"
			href="https://{site.domain}"
			target="_blank"
			rel="noopener noreferrer"
		>
			{site.domain}
		</a>
	</div>

	<RangePicker
		range={current}
		{preset}
		onpreset={id => router.open(`${base}?range=${id}`)}
		oncustom={r => router.open(`${base}?from=${r.from}&to=${r.to}`)}
	/>
</div>

{#if isNew}
	<div class="card new">
		<h2>Add the script to every page of {site.domain}</h2>
		<Copy text={site.snippet} />
		<p class="hint">
			Visits show up here right away. Calls of Simple Analytics'
			<code>sa_event</code>
			keep working.
		</p>
	</div>
{/if}

<div class="tiles">
	<div class="card tile">
		<span class="label">Visitors</span>
		<span class="value">{num(stats.visitors)}</span>
	</div>
	<div class="card tile">
		<span class="label">Pageviews</span>
		<span class="value">{num(stats.pageviews)}</span>
	</div>
</div>

<div class="card">
	<Chart buckets={stats.buckets} hourly={stats.hourly} />
</div>

<div class="grid">
	<BarList
		title="Pages"
		unit="Pageviews"
		items={stats.pages}
		value={p => p.pageviews}
	>
		{#snippet label(p)}{p.path}{/snippet}
	</BarList>

	<BarList
		title="Referrers"
		unit="Visitors"
		items={stats.referrers}
		value={r => r.visitors}
	>
		{#snippet label(r)}
			{#if r.referrer}{r.referrer}{:else}<span class="muted">
					Direct or unknown
				</span>{/if}
		{/snippet}
	</BarList>
</div>

<div class="grid">
	<BarList
		title="Keys"
		unit="Count"
		items={stats.keys}
		value={k => k.count}
		empty="No keys in this time."
		onselect={k => selectKey(k.key)}
		selected={k => k.key === key}
	>
		{#snippet label(k)}{k.key}{/snippet}
	</BarList>

	{#if key}
		<div class="values">
			{#if valuesError}
				<p class="error">{valuesError}</p>
			{:else if !fields.length}
				<div class="card">
					<h2>{key}</h2>
					<p class="muted">No data sent with this key.</p>
				</div>
			{/if}
			{#each fields as [field, counts] (field)}
				<BarList
					title="{key}: {field}"
					unit="Count"
					items={counts}
					value={v => v.count}
				>
					{#snippet label(v)}{v.value}{/snippet}
				</BarList>
			{/each}
		</div>
	{:else}
		<div class="card hint">
			Click a key to see the values of its data. Send keys with
			<pre class="code">{keyExample}</pre>
		</div>
	{/if}
</div>

{#if $user?.admin}
	<div class="settings">
		{#if !showSettings}
			<Button kind="secondary" onclick={() => (showSettings = true)}>
				Settings
			</Button>
		{:else}
			<div class="card">
				<h2>Script</h2>
				<Copy text={site.snippet} />

				<h2 class="space">Settings</h2>
				<form class="form" onsubmit={save}>
					<Field label="Domain" bind:value={domain} required />
					<Field label="Name" bind:value={name} required />
					{#if settingsError}
						<p class="error">{settingsError}</p>
					{/if}
					<div class="row-buttons">
						<Button type="submit">Save</Button>
						<Button kind="danger" onclick={remove}>
							Delete site
						</Button>
					</div>
				</form>
			</div>
		{/if}
	</div>
{/if}

<style lang="scss">
	.title {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: flex-start;
		gap: 1rem;
		margin-bottom: 1.5rem;

		h1 {
			margin-bottom: 0.1rem;
		}
	}

	.domain {
		color: var(--ink-2);
		font-size: 0.9rem;
	}

	.new {
		margin-bottom: 1.5rem;
		border-color: var(--accent);

		.hint {
			margin-top: 0.75rem;
		}
	}

	.tiles {
		display: flex;
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.tile {
		display: grid;
		min-width: 10rem;

		.label {
			font-size: 0.85rem;
			color: var(--ink-2);
		}

		.value {
			font-size: 2rem;
			font-weight: 700;
		}
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(20rem, 1fr));
		gap: 1rem;
		margin-top: 1rem;
		align-items: start;
	}

	.values {
		display: grid;
		gap: 1rem;
	}

	.hint pre {
		margin-top: 0.5rem;
	}

	code {
		font-family: ui-monospace, 'SF Mono', Menlo, monospace;
		font-size: 0.85em;
	}

	.settings {
		margin-top: 2rem;
	}

	.space {
		margin-top: 1.5rem;
	}
</style>
