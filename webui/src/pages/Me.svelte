<script lang="ts">
	import { changePassword, logout } from '@/api/users';
	import Button from '@/components/Button.svelte';
	import Field from '@/components/Field.svelte';
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

	<h2 class="space">Theme</h2>
	<div class="themes" role="group" aria-label="Theme">
		{#each THEMES as t (t.id)}
			<button
				class:active={t.id === theme}
				aria-pressed={t.id === theme}
				onclick={() => chooseTheme(t.id)}
			>
				{t.label}
			</button>
		{/each}
	</div>
	<p class="hint">Saved in this browser.</p>

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

	// like the presets of the range picker
	.themes {
		display: inline-flex;
		margin-bottom: 0.4rem;
		background-color: var(--surface);
		border: 2px solid var(--border);
		border-radius: var(--radius);
		box-shadow: 3px 3px 0 var(--shadow);
		overflow: hidden;

		button {
			padding: 0.35rem 0.9rem;
			font-family: var(--mono);
			font-size: 0.78rem;
			font-weight: 600;
			color: var(--ink-2);
			background: none;
			border: none;
			cursor: pointer;

			&:hover {
				color: var(--ink);
				background-color: var(--wash);
			}

			&.active {
				color: var(--marker-ink);
				background-color: var(--marker);
			}

			& + button {
				border-left: 2px solid var(--border);
			}
		}
	}
</style>
