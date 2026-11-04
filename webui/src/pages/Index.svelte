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
	import Sparkline from '@/components/Sparkline.svelte';
	import { num } from '@/lib/format';
	import { getRouter, getSession } from '@/main';

	const { sites }: { sites: SiteOverview[] } = $props();

	const router = getRouter();
	const user = getSession().user;

	const sum = (s: SiteOverview, days: number) =>
		s.days.slice(-days).reduce((acc, d) => acc + d.visitors, 0);

	let adding = $state(false);
	let domain = $state('');
	let name = $state('');
	let error = $state('');

	async function add(e: SubmitEvent) {
		e.preventDefault();
		error = '';

		try {
			const site = await createSite(domain, name || domain);
			router.open(`/sites/${site.id}?new=1`);
		} catch (e: any) {
			error = e.message;
		}
	}
</script>

<div class="title">
	<h1>Sites</h1>
	{#if $user?.admin && !adding}
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
		{$user?.admin
			? 'No sites yet, add the first one.'
			: 'You have no sites yet, ask an admin to give you some.'}
	</p>
{:else}
	<div class="card">
		<table class="list sites">
			<thead>
				<tr>
					<th>Site</th>
					<th class="spark">Visitors, 30 days</th>
					<th class="right">Today</th>
					<th class="right">7 days</th>
					<th class="right">30 days</th>
				</tr>
			</thead>
			<tbody>
				{#each sites as site (site.id)}
					<tr>
						<td>
							<a href="/sites/{site.id}" class="name">
								{site.name}
							</a>
							<div class="muted domain">{site.domain}</div>
						</td>
						<td class="spark">
							<Sparkline days={site.days} />
						</td>
						<td class="right num">{num(sum(site, 1))}</td>
						<td class="right num">{num(sum(site, 7))}</td>
						<td class="right num strong">{num(sum(site, 30))}</td>
					</tr>
				{/each}
			</tbody>
		</table>
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

	.sites {
		td {
			padding-top: 0.7rem;
			padding-bottom: 0.7rem;
		}

		tr:last-child td {
			border-bottom: none;
		}
	}

	.name {
		font-weight: 600;
		color: var(--ink);
		text-decoration: none;

		&:hover {
			text-decoration: underline;
		}
	}

	.domain {
		font-size: 0.85rem;
	}

	.strong {
		font-weight: 600;
	}

	@media (max-width: 700px) {
		.spark {
			display: none;
		}
	}
</style>
