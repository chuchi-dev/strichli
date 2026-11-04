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
	import { timestamp } from '@/lib/format';

	const { tokens: loaded }: { tokens: ApiToken[] } = $props();

	// svelte-ignore state_referenced_locally
	let tokens = $state(loaded);
	let name = $state('');
	let secret = $state('');
	let error = $state('');

	async function create(e: SubmitEvent) {
		e.preventDefault();
		error = '';

		try {
			const token = await createToken(name);
			secret = token.secret;
			tokens = [...tokens, token];
			name = '';
		} catch (e: any) {
			error = e.message;
		}
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
	const example = $derived(
		`curl ${origin}/api/sites \\\n` +
			`  -H 'authorization: Bearer ${secret || '<token>'}' \\\n` +
			`  -H 'content-type: application/json' \\\n` +
			`  -d '{"domain": "example.com", "name": "Example"}'`,
	);
</script>

<h1>API tokens</h1>

<p class="hint intro">
	A token acts as you on the sites api, for scripts and AI agents: list sites
	and their stats, add and change sites. Deleting a site needs the webui.
</p>

<div class="card">
	<form class="form" onsubmit={create}>
		<Field label="Name" bind:value={name} placeholder="noah" required />
		{#if error}
			<p class="error">{error}</p>
		{/if}
		<div>
			<Button type="submit">Create token</Button>
		</div>
	</form>

	{#if secret}
		<div class="secret">
			<h2>The new token, it is only shown now</h2>
			<Copy text={secret} />
		</div>
	{/if}

	<h2 class="space">Add a site</h2>
	<pre class="code">{example}</pre>
	<p class="hint">
		Also: <code>GET /api/sites</code>
		,
		<code>
			GET /api/sites/&#123;id&#125;/stats?from=2026-11-01&to=2026-11-30
		</code>
		,
		<code>PUT /api/sites/&#123;id&#125;</code>
		.
	</p>
</div>

{#if tokens.length}
	<div class="card list-card">
		<table class="list">
			<thead>
				<tr>
					<th>Name</th>
					<th>Created</th>
					<th>Last used</th>
					<th></th>
				</tr>
			</thead>
			<tbody>
				{#each tokens as token (token.id)}
					<tr>
						<td>{token.name}</td>
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

	.secret {
		margin-top: 1.5rem;
	}

	.space {
		margin-top: 1.5rem;
	}

	.hint {
		margin-top: 0.5rem;
	}

	code {
		font-family: ui-monospace, 'SF Mono', Menlo, monospace;
		font-size: 0.85em;
	}

	.list-card {
		margin-top: 1rem;
	}
</style>
