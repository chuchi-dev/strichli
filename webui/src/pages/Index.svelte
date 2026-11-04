<script module lang="ts">
	import { listSites } from '@/api/sites';

	export async function loadProps() {
		return { sites: await listSites() };
	}
</script>

<script lang="ts">
	import type { SiteOverview } from '@/api/sites';
	import { createSite } from '@/api/sites';
	import Button from '@/components/Button.svelte';
	import Field from '@/components/Field.svelte';
	import SiteCard from '@/components/SiteCard.svelte';
	import { go } from '@/lib/router.svelte';
	import { session } from '@/lib/session.svelte';

	const { sites }: { sites: SiteOverview[] } = $props();

	const visitors = (s: SiteOverview) =>
		s.days.reduce((acc, d) => acc + d.visitors, 0);

	let filter = $state('');
	let sort: 'visitors' | 'name' = $state('visitors');

	const shown = $derived(
		sites
			.filter(s => {
				const f = filter.trim().toLowerCase();
				return (
					!f ||
					s.name.toLowerCase().includes(f) ||
					s.domain.includes(f)
				);
			})
			.toSorted((a, b) =>
				sort === 'name'
					? a.name.localeCompare(b.name)
					: visitors(b) - visitors(a),
			),
	);

	let adding = $state(false);
	let domain = $state('');
	let name = $state('');
	let error = $state('');

	async function add(e: SubmitEvent) {
		e.preventDefault();
		error = '';

		try {
			const site = await createSite(domain, name || domain);
			go(`/sites/${site.id}?new=1`);
		} catch (e: any) {
			error = e.message;
		}
	}
</script>

<div class="title">
	<h1>Sites</h1>
	{#if session.user?.admin && !adding}
		<Button onclick={() => (adding = true)}>Add site</Button>
	{/if}
</div>

{#if adding}
	<div class="card add">
		<h2>Add a site</h2>
		<form class="form" onsubmit={add}>
			<Field
				label="Domain"
				bind:value={domain}
				placeholder="example.com"
				hint="Visits on www. count too."
				required
			/>
			<Field label="Name" bind:value={name} placeholder="Example" />
			{#if error}
				<p class="error">{error}</p>
			{/if}
			<div class="row-buttons">
				<Button type="submit">Add</Button>
				<Button kind="secondary" onclick={() => (adding = false)}>
					Cancel
				</Button>
			</div>
		</form>
	</div>
{/if}

{#if !sites.length}
	<p class="hint">
		{session.user?.admin
			? 'No sites yet, add the first one.'
			: 'You have no sites yet, ask an admin to give you some.'}
	</p>
{:else}
	<div class="tools">
		<input
			type="search"
			placeholder="Filter sites"
			aria-label="Filter sites"
			bind:value={filter}
			spellcheck="false"
		/>
		<div class="legend">
			<span class="key visitors"></span>
			Visitors
			<span class="key pageviews"></span>
			Pageviews, last 30 days
		</div>
		<label class="sort">
			Sort by
			<select bind:value={sort}>
				<option value="visitors">visitors</option>
				<option value="name">name</option>
			</select>
		</label>
	</div>

	<div class="grid">
		{#each shown as site (site.id)}
			<SiteCard {site} />
		{:else}
			<p class="muted">No site matches.</p>
		{/each}
	</div>
{/if}

<style lang="scss">
	.title {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
	}

	.add {
		margin-bottom: 1.5rem;
	}

	.tools {
		display: flex;
		flex-wrap: wrap;
		gap: 1rem;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 1rem;

		input {
			width: 16rem;
		}
	}

	.legend {
		display: flex;
		gap: 0.4rem;
		align-items: center;
		font-family: var(--mono);
		font-size: 0.78rem;
		color: var(--ink-2);

		.key {
			width: 14px;
			height: 0;
			border-top: 2px solid var(--series-1);

			&.pageviews {
				margin-left: 0.6rem;
				border-top: 2px dashed var(--series-2);
			}
		}
	}

	.sort {
		font-family: var(--mono);
		font-size: 0.78rem;
		color: var(--ink-2);

		select {
			margin-left: 0.3rem;
			padding: 0.25rem 0.4rem;
		}
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr));
		gap: 1.5rem;
	}
</style>
