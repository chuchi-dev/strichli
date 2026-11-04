<script module lang="ts">
	import type LoadProps from '@/lib/LoadProps';

	export const isPublic = true;

	export async function loadProps(_props: object, lp: LoadProps) {
		if (lp.session.isLoggedIn()) lp.setRedirect('/');
	}
</script>

<script lang="ts">
	import Button from '@/components/Button.svelte';
	import Field from '@/components/Field.svelte';
	import { login } from '@/api/users';
	import { getRouter, getSession } from '@/main';

	const router = getRouter();
	const session = getSession();

	let email = $state('');
	let password = $state('');
	let error = $state('');
	let busy = $state(false);

	async function submit(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		busy = true;

		try {
			session.setAuthed(await login(email, password));
			router.open('/');
		} catch (e: any) {
			error = e.message;
		} finally {
			busy = false;
		}
	}
</script>

<div class="login card">
	<h1>Login</h1>

	<form class="form" onsubmit={submit}>
		<Field
			label="Email"
			type="email"
			bind:value={email}
			autocomplete="username"
			required
		/>
		<Field
			label="Password"
			type="password"
			bind:value={password}
			autocomplete="current-password"
			required
		/>

		{#if error}
			<p class="error">{error}</p>
		{/if}

		<div>
			<Button type="submit" disabled={busy}>Login</Button>
		</div>
	</form>
</div>

<style lang="scss">
	.login {
		max-width: 24rem;
		margin: 3rem auto 0;
	}
</style>
