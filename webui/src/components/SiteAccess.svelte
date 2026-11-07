<script lang="ts">
	import type { UserWithSites } from '@/api/users';
	import { createUser, updateUser } from '@/api/users';
	import Button from './Button.svelte';
	import Copy from './Copy.svelte';
	import Field from './Field.svelte';
	import Form from './Form.svelte';

	// who sees a site: admins see every site, other users the sites they
	// were given. A client gets a login for their site here.
	const {
		siteId,
		users: loaded,
	}: { siteId: string; users: UserWithSites[] } = $props();

	// svelte-ignore state_referenced_locally
	let users = $state(loaded);
	let name = $state('');
	let email = $state('');
	/** of the user just invited, where they set their password */
	let link = $state('');
	let existing = $state('');
	let error = $state('');

	const admins = $derived(users.filter(u => u.admin));
	const members = $derived(
		users.filter(u => !u.admin && u.siteIds.includes(siteId)),
	);
	const others = $derived(
		users.filter(u => !u.admin && !u.siteIds.includes(siteId)),
	);

	function replace(user: UserWithSites) {
		users = users.map(u => (u.id === user.id ? user : u));
	}

	/** the same user with other sites, the rest stays */
	function withSites(user: UserWithSites, siteIds: string[]) {
		return updateUser(user.id, {
			email: user.email,
			name: user.name,
			admin: user.admin,
			siteIds,
		});
	}

	async function invite() {
		const user = await createUser({
			email,
			name,
			admin: false,
			siteIds: [siteId],
		});
		users = [...users, user];
		link = user.passwordLink ?? '';
		name = '';
		email = '';
	}

	async function grant() {
		const user = others.find(u => u.id === existing);
		if (!user) return;

		error = '';
		try {
			replace(await withSites(user, [...user.siteIds, siteId]));
			existing = '';
		} catch (e: any) {
			error = e.message;
		}
	}

	async function revoke(user: UserWithSites) {
		if (!confirm(`${user.name} won't see this site anymore.`)) return;

		error = '';
		try {
			replace(
				await withSites(
					user,
					user.siteIds.filter(id => id !== siteId),
				),
			);
		} catch (e: any) {
			error = e.message;
		}
	}
</script>

<table class="list">
	<tbody>
		{#each admins as user (user.id)}
			<tr>
				<td>{user.name}</td>
				<td class="muted">{user.email}</td>
				<td class="right muted">admin, sees every site</td>
			</tr>
		{/each}
		{#each members as user (user.id)}
			<tr>
				<td>{user.name}</td>
				<td class="muted">{user.email}</td>
				<td class="right">
					<Button kind="danger" small onclick={() => revoke(user)}>
						Remove
					</Button>
				</td>
			</tr>
		{/each}
	</tbody>
</table>

{#if error}
	<p class="error">{error}</p>
{/if}

{#if others.length}
	<div class="grant">
		<select bind:value={existing} aria-label="Existing user">
			<option value="">Give an existing user access</option>
			{#each others as user (user.id)}
				<option value={user.id}>{user.name} ({user.email})</option>
			{/each}
		</select>
		<Button kind="secondary" onclick={grant} disabled={!existing}>
			Give access
		</Button>
	</div>
{/if}

<h3>Invite someone, e.g. the client</h3>
<p class="hint">
	They only see this site. You get a link (valid 7 days) with which they set
	their password.
</p>
<Form onsubmit={invite} submit="Invite">
	<Field label="Name" bind:value={name} required />
	<Field label="Email" type="email" bind:value={email} required />
</Form>

{#if link}
	<div class="link">
		<p class="hint">Send them this link:</p>
		<Copy text={link} />
	</div>
{/if}

<style lang="scss">
	.grant {
		display: flex;
		flex-wrap: wrap;
		gap: 0.6rem;
		align-items: center;
		margin-top: 1rem;
	}

	h3 {
		margin-top: 1.5rem;
		margin-bottom: 0.3rem;
		font-weight: 700;
	}

	.hint {
		margin-bottom: 0.75rem;
	}

	.link {
		margin-top: 1rem;
	}

	tr:last-child td {
		border-bottom: none;
	}
</style>
