<script lang="ts">
	import { changePassword, logout } from '@/api/users';
	import Button from '@/components/Button.svelte';
	import Field from '@/components/Field.svelte';
	import Form from '@/components/Form.svelte';
	import Segmented from '@/components/Segmented.svelte';
	import { go } from '@/lib/router.svelte';
	import { session } from '@/lib/session.svelte';
	import { getTheme, setTheme, type Theme } from '@/lib/theme';

	const THEMES: { id: Theme; label: string }[] = [
		{ id: 'system', label: 'System' },
		{ id: 'light', label: 'Light' },
		{ id: 'dark', label: 'Dark' },
	];

	let theme = $state(getTheme());

	function chooseTheme(t: Theme) {
		theme = t;
		setTheme(t);
	}

	let password = $state('');
	let newPassword = $state('');

	async function submit() {
		await changePassword(password, newPassword);
		password = '';
		newPassword = '';
	}

	async function onLogout() {
		try {
			await logout();
		} catch (e) {
			console.error('logout failed', e);
		}
		session.logout();
		go('/login');
	}
</script>

<h1>{session.user?.name}</h1>

<div class="card">
	<p>
		{session.user?.email}
		<span class="muted">{session.user?.admin ? '(admin)' : ''}</span>
	</p>

	<h2 class="space-top">Theme</h2>
	<Segmented
		label="Theme"
		options={THEMES}
		value={theme}
		onchange={chooseTheme}
	/>
	<p class="hint">Saved in this browser.</p>

	<h2 class="space-top">Change the password</h2>
	<Form onsubmit={submit} submit="Change password" done="Changed">
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
	</Form>

	<div class="space-top">
		<Button kind="secondary" onclick={onLogout}>Logout</Button>
	</div>
</div>

<style lang="scss">
	.hint {
		margin-top: 0.5rem;
	}
</style>
