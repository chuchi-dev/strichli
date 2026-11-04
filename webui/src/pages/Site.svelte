<script module lang="ts">
	import { getEvents, getSite, getStats } from '@/api/sites';
	import { rangeFromProps } from '@/lib/range';

	type Props = {
		id: string;
		tab?: string;
		range?: string;
		from?: string;
		to?: string;
	};

	const MAX_COLUMNS = 5;

	export async function loadProps(props: Props) {
		const { range, preset } = rangeFromProps(props);
		const tab =
			props.tab === 'events' || props.tab === 'settings'
				? props.tab
				: 'visitors';
		const site = await getSite(props.id);
		const base = { site, current: range, preset, tab };

		if (tab === 'events') {
			// the fields come with the first answer, show them as columns
			let events = await getEvents(props.id, range, []);
			const columns = events.fields.slice(0, MAX_COLUMNS);
			if (columns.length)
				events = await getEvents(props.id, range, columns);
			return { ...base, events, columns };
		}

		if (tab === 'visitors') {
			return { ...base, stats: await getStats(props.id, range) };
		}

		return base;
	}
</script>

<script lang="ts">
	import type { EventTable, Range, Site, Stats } from '@/api/sites';
	import { deleteSite, updateSite } from '@/api/sites';
	import BarList from '@/components/BarList.svelte';
	import Button from '@/components/Button.svelte';
	import Chart from '@/components/Chart.svelte';
	import Copy from '@/components/Copy.svelte';
	import EventsView from '@/components/EventsView.svelte';
	import Field from '@/components/Field.svelte';
	import RangePicker from '@/components/RangePicker.svelte';
	import { num } from '@/lib/format';
	import { getRouter, getSession } from '@/main';

	type Loaded = {
		site: Site;
		current: Range;
		preset: string | null;
		tab: 'visitors' | 'events' | 'settings';
		stats?: Stats;
		events?: EventTable;
		columns?: string[];
		/** just added, show how to include the script */
		new?: string;
	};

	const {
		site,
		current,
		preset,
		tab,
		stats,
		events,
		columns = [],
		new: isNew,
	}: Loaded = $props();

	const router = getRouter();
	const user = getSession().user;

	$effect(() => {
		document.title = `${site.name} - strichli`;
	});

	/** the page with another tab or range, the rest stays */
	function link(opts: { tab?: string; range?: string; custom?: Range }) {
		// eslint-disable-next-line svelte/prefer-svelte-reactivity -- a local value, not state
		const params = new URLSearchParams();
		const t = opts.tab ?? tab;
		if (t !== 'visitors') params.set('tab', t);

		if (opts.custom) {
			params.set('from', opts.custom.from);
			params.set('to', opts.custom.to);
		} else if (opts.range ?? preset) {
			params.set('range', (opts.range ?? preset)!);
		} else {
			params.set('from', current.from);
			params.set('to', current.to);
		}

		return `/sites/${site.id}?${params}`;
	}

	const TABS = [
		{ id: 'visitors', label: 'Visitors' },
		{ id: 'events', label: 'Events' },
		{ id: 'settings', label: 'Settings', admin: true },
	];

	// settings
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
			router.open(link({}));
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

	{#if tab !== 'settings'}
		<RangePicker
			range={current}
			{preset}
			onpreset={id => router.open(link({ range: id }))}
			oncustom={r => router.open(link({ custom: r }))}
		/>
	{/if}
</div>

<nav class="tabs">
	{#each TABS.filter(t => !t.admin || $user?.admin) as t (t.id)}
		<a
			href={link({ tab: t.id })}
			class:active={t.id === tab}
			aria-current={t.id === tab ? 'page' : undefined}
		>
			{t.label}
		</a>
	{/each}
</nav>

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

{#if tab === 'visitors' && stats}
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

		<BarList
			title="Events"
			unit="Count"
			items={stats.keys}
			value={k => k.count}
			empty="No events in this time."
		>
			{#snippet label(k)}{k.key}{/snippet}
		</BarList>
	</div>

	{#if stats.keys.length}
		<p class="more">
			<a href={link({ tab: 'events' })}>All events with their data →</a>
		</p>
	{/if}
{:else if tab === 'events' && events}
	<EventsView siteId={site.id} range={current} table={events} {columns} />
{:else if tab === 'settings'}
	<div class="card">
		<h2>Script</h2>
		<Copy text={site.snippet} />
		<p class="hint">
			Send events with <code>{keyExample}</code>
		</p>

		<h2 class="space">Site</h2>
		<form class="form" onsubmit={save}>
			<Field label="Domain" bind:value={domain} required />
			<Field label="Name" bind:value={name} required />
			{#if settingsError}
				<p class="error">{settingsError}</p>
			{/if}
			<div class="row-buttons">
				<Button type="submit">Save</Button>
				<Button kind="danger" onclick={remove}>Delete site</Button>
			</div>
		</form>
	</div>
{/if}

<style lang="scss">
	.title {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: flex-start;
		gap: 1rem;
		margin-bottom: 1rem;

		h1 {
			margin-bottom: 0.1rem;
		}
	}

	.domain {
		font-family: var(--mono);
		font-size: 0.85rem;
		color: var(--ink-2);
	}

	.tabs {
		display: flex;
		gap: 0.25rem;
		margin-bottom: 1.75rem;
		border-bottom: 2px solid var(--border);

		a {
			padding: 0.5rem 0.9rem;
			margin-bottom: -2px;
			font-family: var(--mono);
			font-size: 0.82rem;
			font-weight: 600;
			letter-spacing: 0.05em;
			text-transform: uppercase;
			color: var(--ink-2);
			text-decoration: none;
			border-bottom: 5px solid transparent;

			&:hover {
				color: var(--ink);
			}

			&.active {
				color: var(--ink);
				border-bottom-color: var(--marker);
			}
		}
	}

	.new {
		margin-bottom: 1.5rem;
		box-shadow: 4px 4px 0 var(--marker);

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
		min-width: 11rem;

		.label {
			font-family: var(--mono);
			font-size: 0.72rem;
			font-weight: 600;
			letter-spacing: 0.06em;
			text-transform: uppercase;
			color: var(--muted);
		}

		.value {
			font-size: 2.6rem;
			font-weight: 800;
			letter-spacing: -0.02em;
			line-height: 1.1;
		}
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr));
		gap: 1rem;
		margin-top: 1rem;
		align-items: start;
	}

	.more {
		margin-top: 1rem;
		text-align: right;
		font-family: var(--mono);
		font-size: 0.85rem;
	}

	.hint {
		margin-top: 0.5rem;
	}

	code {
		font-size: 0.85em;
	}

	.space {
		margin-top: 1.5rem;
	}
</style>
