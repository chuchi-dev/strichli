<script module lang="ts">
	import type { LoadProps } from '@/lib/router.svelte';
	import { session } from '@/lib/session.svelte';

	export const isPublic = true;

	export async function loadProps(_props: object, lp: LoadProps) {
		if (session.user) lp.setRedirect('/');
	}
</script>

<script lang="ts">
	import { login } from '@/api/users';
	import Field from '@/components/Field.svelte';
	import Form from '@/components/Form.svelte';
	import { go } from '@/lib/router.svelte';

	let email = $state('');
	let password = $state('');

	async function submit() {
		const resp = await login(email, password);
		session.login(resp.token, resp.user);
		// back to the page that asked for the login
		const url = new URLSearchParams(location.search).get('url');
		go(url?.startsWith('/') ? url : '/');
	}
</script>

<div class="card narrow">
	<h1>Login</h1>

	<Form onsubmit={submit} submit="Login">
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
	</Form>
</div>
