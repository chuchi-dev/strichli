<script module lang="ts">
	import { passwordLinkUser } from '@/api/users';

	export const isPublic = true;

	export async function loadProps(props: { token?: string }) {
		try {
			return { user: await passwordLinkUser(props.token ?? '') };
		} catch (_e) {
			return { user: null };
		}
	}
</script>

<script lang="ts">
	import { setPasswordWithLink } from '@/api/users';
	import Button from '@/components/Button.svelte';
	import Field from '@/components/Field.svelte';
	import { go } from '@/lib/router.svelte';
	import { session } from '@/lib/session.svelte';

	type Props = {
		token?: string;
		user: { email: string; name: string } | null;
	};

	const { token = '', user }: Props = $props();

	let password = $state('');
	let repeat = $state('');
	let error = $state('');
	let busy = $state(false);

	async function submit(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		if (password !== repeat) {
			error = 'The passwords are not the same';
			return;
		}

		busy = true;
		try {
			session.login(await setPasswordWithLink(token, password));
			go('/');
		} catch (e: any) {
			error = e.message;
		} finally {
			busy = false;
		}
	}
</script>

<div class="password card">
	<h1>Set the password</h1>

	{#if !user}
		<p class="error">
			This link is invalid, used or expired. Ask an admin for a new one.
		</p>
	{:else}
		<p class="hint who">{user.name}, {user.email}</p>

		<form class="form" onsubmit={submit}>
			<input
				type="email"
				value={user.email}
				autocomplete="username"
				hidden
				readonly
			/>
			<Field
				label="Password"
				type="password"
				bind:value={password}
				autocomplete="new-password"
				hint="At least 10 characters."
				required
			/>
			<Field
				label="Repeat it"
				type="password"
				bind:value={repeat}
				autocomplete="new-password"
				required
			/>

			{#if error}
				<p class="error">{error}</p>
			{/if}

			<div>
				<Button type="submit" disabled={busy}>Set and log in</Button>
			</div>
		</form>
	{/if}
</div>

<style lang="scss">
	.password {
		max-width: 24rem;
		margin: 3rem auto 0;
	}

	.who {
		margin-bottom: 1rem;
	}
</style>
