<script lang="ts">
	import { changePassword, logout } from '@/api/users';
	import Button from '@/components/Button.svelte';
	import Field from '@/components/Field.svelte';
	import { getRouter, getSession } from '@/main';

	const router = getRouter();
	const session = getSession();
	const user = session.user;

	let password = $state('');
	let newPassword = $state('');
	let error = $state('');
	let done = $state(false);

	async function submit(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		done = false;

		try {
			await changePassword(password, newPassword);
			password = '';
			newPassword = '';
			done = true;
		} catch (e: any) {
			error = e.message;
		}
	}

	async function onLogout() {
		try {
			await logout();
		} catch (e) {
			console.error('logout failed', e);
		}
		session.invalidate();
		router.open('/login');
	}
</script>

<h1>{$user?.name}</h1>

<div class="card">
	<p>
		{$user?.email}
		<span class="muted">{$user?.admin ? '(admin)' : ''}</span>
	</p>

	<h2 class="space">Change the password</h2>
	<form class="form" onsubmit={submit}>
		<Field
			label="Current password"
			type="password"
			bind:value={password}
			autocomplete="current-password"
			required
		/>
		<Field
			label="New password"
			type="password"
			bind:value={newPassword}
			autocomplete="new-password"
			hint="At least 10 characters, logs out your other sessions."
			required
		/>
		{#if error}
			<p class="error">{error}</p>
		{/if}
		{#if done}
			<p class="ok">Saved</p>
		{/if}
		<div>
			<Button type="submit">Change password</Button>
		</div>
	</form>

	<div class="space">
		<Button kind="secondary" onclick={onLogout}>Logout</Button>
	</div>
</div>

<style lang="scss">
	.space {
		margin-top: 1.5rem;
	}

	.ok {
		color: var(--ok);
	}
</style>
