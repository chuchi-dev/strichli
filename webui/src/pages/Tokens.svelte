<script module lang="ts">
	import { listTokens } from '@/api/users';

	export async function loadProps() {
		return { tokens: await listTokens() };
	}
</script>

<script lang="ts">
	import type { ApiToken } from '@/api/users';
	import { createToken, deleteToken } from '@/api/users';
	import Button from '@/components/Button.svelte';
	import Copy from '@/components/Copy.svelte';
	import Field from '@/components/Field.svelte';
	import Form from '@/components/Form.svelte';
	import { timestamp } from '@/lib/format';
	import { session } from '@/lib/session.svelte';

	const { tokens: loaded }: { tokens: ApiToken[] } = $props();

	// svelte-ignore state_referenced_locally
	let tokens = $state(loaded);
	let name = $state('');
	let write = $state(false);
	let secret = $state('');
	let error = $state('');

	async function create() {
		const token = await createToken(name, write);
		secret = token.secret;
		tokens = [...tokens, token];
		name = '';
		write = false;
	}

	async function remove(token: ApiToken) {
		if (!confirm(`Delete the token ${token.name}?`)) return;

		try {
			await deleteToken(token.id);
			tokens = tokens.filter(t => t.id !== token.id);
		} catch (e: any) {
			error = e.message;
		}
	}

	const origin = location.origin;
	const auth = $derived(
		`  -H 'authorization: Bearer ${secret || '<token>'}'`,
	);
	const readExample = $derived(
		`curl ${origin}/api/sites \\\n${auth}\n\n` +
			`curl '${origin}/api/sites/<id>/stats?range=30d' \\\n${auth}`,
	);
	const example = $derived(
		`curl ${origin}/api/sites \\\n` +
			`  -H 'authorization: Bearer ${secret || '<token>'}' \\\n` +
			`  -H 'content-type: application/json' \\\n` +
			`  -d '{"domain": "example.com", "name": "Example"}'`,
	);
</script>

<h1>API tokens</h1>

<p class="hint intro">
	{#if session.user?.admin}
		A token reads the stats of all sites, for scripts, spreadsheets or AI
		agents. One that can write also adds and changes sites. Deleting a site
		needs the webui.
	{:else}
		A token reads the stats of your sites, for scripts, spreadsheets or AI
		agents. It can't change anything.
	{/if}
</p>

<div class="card">
	<Form onsubmit={create} submit="Create token">
		<Field
			label="Name"
			bind:value={name}
			placeholder="spreadsheet"
			required
		/>
		{#if session.user?.admin}
			<label class="check">
				<input type="checkbox" bind:checked={write} />
				Can write: add and change sites
			</label>
		{/if}
	</Form>

	{#if secret}
		<div class="secret">
			<h2>The new token, it is only shown now</h2>
			<Copy text={secret} />
		</div>
	{/if}

	<h2 class="space-top">Read the stats</h2>
	<pre class="code">{readExample}</pre>
	<p class="hint">
		<code>range</code>
		is
		<code>today</code>
		,
		<code>7d</code>
		,
		<code>30d</code>
		,
		<code>90d</code>
		or
		<code>12m</code>
		, or use
		<code>from=2026-11-01&to=2026-11-30</code>
		. The events of a site:
		<code>/api/sites/&#123;id&#125;/events?range=30d&fields=domain</code>
		.
	</p>

	{#if session.user?.admin}
		<h2 class="space-top">Add a site, with a token that can write</h2>
		<pre class="code">{example}</pre>
		<p class="hint">
			Change one with <code>PUT /api/sites/&#123;id&#125;</code>
			.
		</p>
	{/if}
</div>

{#if error}
	<p class="error list-card">{error}</p>
{/if}

{#if tokens.length}
	<div class="card list-card">
		<table class="list">
			<thead>
				<tr>
					<th>Name</th>
					{#if session.user?.admin}
						<th>Access</th>
					{/if}
					<th>Created</th>
					<th>Last used</th>
					<th></th>
				</tr>
			</thead>
			<tbody>
				{#each tokens as token (token.id)}
					<tr>
						<td>{token.name}</td>
						{#if session.user?.admin}
							<td>
								{token.canWrite ? 'Read and write' : 'Read'}
							</td>
						{/if}
						<td>{timestamp(token.createdOn)}</td>
						<td>
							{token.lastUsed
								? timestamp(token.lastUsed)
								: 'Never'}
						</td>
						<td class="right">
							<Button kind="danger" onclick={() => remove(token)}>
								Delete
							</Button>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}

<style lang="scss">
	.intro {
		max-width: 40rem;
		margin-bottom: 1.5rem;
	}

	.check {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.secret {
		margin-top: 1.5rem;
	}

	.hint {
		margin-top: 0.5rem;
	}

	.list-card {
		margin-top: 1rem;
	}
</style>
