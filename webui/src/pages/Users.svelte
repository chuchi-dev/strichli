<script module lang="ts">
	import { listSites } from '@/api/sites';
	import { listUsers } from '@/api/users';

	export async function loadProps() {
		const [users, sites] = await Promise.all([listUsers(), listSites()]);
		return { users, sites };
	}
</script>

<script lang="ts">
	import type { SiteOverview } from '@/api/sites';
	import type { UserWithSites } from '@/api/users';
	import { createUser, deleteUser, updateUser } from '@/api/users';
	import Button from '@/components/Button.svelte';
	import Copy from '@/components/Copy.svelte';
	import Field from '@/components/Field.svelte';
	import Form from '@/components/Form.svelte';
	import { session } from '@/lib/session.svelte';

	const {
		users: loaded,
		sites,
	}: { users: UserWithSites[]; sites: SiteOverview[] } = $props();

	// svelte-ignore state_referenced_locally
	let users = $state(loaded);

	/** the user in the form, null for a new one */
	let editing: UserWithSites | null = $state(null);
	let open = $state(false);
	let email = $state('');
	let name = $state('');
	let password = $state('');
	let admin = $state(false);
	let siteIds: string[] = $state([]);
	let error = $state('');
	/** of the user just added without a password */
	let link = $state('');

	function edit(user: UserWithSites | null) {
		editing = user;
		email = user?.email ?? '';
		name = user?.name ?? '';
		password = '';
		admin = user?.admin ?? false;
		siteIds = [...(user?.siteIds ?? [])];
		link = '';
		open = true;
	}

	const siteNames = (user: UserWithSites) =>
		user.admin
			? 'All sites'
			: sites
					.filter(s => user.siteIds.includes(s.id))
					.map(s => s.name)
					.join(', ') || 'No sites';

	/** the list changes in place, the page stays */
	async function save() {
		const req = {
			email,
			name,
			password: password || undefined,
			admin,
			siteIds: admin ? [] : siteIds,
		};

		if (editing) {
			const user = await updateUser(editing.id, req);
			users = users.map(u => (u.id === user.id ? user : u));
		} else {
			const user = await createUser(req);
			users = [...users, user].toSorted((a, b) =>
				a.name.localeCompare(b.name),
			);
			link = user.passwordLink ?? '';
		}
		open = false;
	}

	async function remove(user: UserWithSites) {
		if (!confirm(`Delete ${user.name} (${user.email})?`)) return;

		error = '';
		try {
			await deleteUser(user.id);
			users = users.filter(u => u.id !== user.id);
		} catch (e: any) {
			error = e.message;
		}
	}
</script>

<div class="title-row">
	<h1>Users</h1>
	{#if !open}
		<Button onclick={() => edit(null)}>Add user</Button>
	{/if}
</div>

{#if open}
	<div class="card edit">
		<h2>{editing ? `Edit ${editing.name}` : 'Add a user'}</h2>
		<Form onsubmit={save} oncancel={() => (open = false)}>
			<Field label="Name" bind:value={name} required />
			<Field label="Email" type="email" bind:value={email} required />
			<Field
				label={editing ? 'New password' : 'Password'}
				type="password"
				bind:value={password}
				autocomplete="new-password"
				hint={editing
					? 'Leave empty to keep the password.'
					: 'At least 10 characters, or empty for a link with which they set it themselves.'}
			/>

			<label class="check">
				<input
					type="checkbox"
					bind:checked={admin}
					disabled={editing?.id === session.user?.id}
				/>
				Admin: sees and manages every site, users and tokens
			</label>

			{#if !admin}
				<fieldset>
					<legend>Sites</legend>
					{#each sites as site (site.id)}
						<label class="check">
							<input
								type="checkbox"
								value={site.id}
								bind:group={siteIds}
							/>
							{site.name}
							<span class="muted">{site.domain}</span>
						</label>
					{:else}
						<p class="muted">No sites yet.</p>
					{/each}
				</fieldset>
			{/if}
		</Form>
	</div>
{/if}

{#if error}
	<p class="error">{error}</p>
{/if}

{#if link}
	<div class="card link">
		<h2>Send this link, it sets the password (valid 7 days)</h2>
		<Copy text={link} />
	</div>
{/if}

<div class="card">
	<table class="list">
		<thead>
			<tr>
				<th>Name</th>
				<th>Email</th>
				<th>Sites</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			{#each users as user (user.id)}
				<tr>
					<td>{user.name}</td>
					<td>{user.email}</td>
					<td>{siteNames(user)}</td>
					<td class="right">
						<div class="row-buttons end">
							<Button kind="secondary" onclick={() => edit(user)}>
								Edit
							</Button>
							{#if user.id !== session.user?.id}
								<Button
									kind="danger"
									onclick={() => remove(user)}
								>
									Delete
								</Button>
							{/if}
						</div>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<style lang="scss">
	@use 'styles/mixins' as *;

	.edit,
	.link,
	.error {
		margin-bottom: 1.5rem;
	}

	.check {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	fieldset {
		display: grid;
		gap: 0.4rem;
		padding: 0.75rem;
		border: 2px solid var(--border);
		border-radius: var(--radius);

		legend {
			@include caps;
			padding: 0 0.3rem;
			color: var(--ink-2);
		}
	}

	.end {
		justify-content: flex-end;
	}
</style>
