<script module lang="ts">
	import { passwordLinkUser } from '@/api/users';
	import { ApiError } from '@/lib/api';

	export const isPublic = true;

	export async function loadProps(props: { token?: string }) {
		try {
			return { user: await passwordLinkUser(props.token ?? '') };
		} catch (e) {
			// only an unknown link is invalid, other errors show as such
			if (e instanceof ApiError && e.type === 'NOT_FOUND') {
				return { user: null };
			}
			throw e;
		}
	}
</script>

<script lang="ts">
	import { setPasswordWithLink } from '@/api/users';
	import Field from '@/components/Field.svelte';
	import Form from '@/components/Form.svelte';
	import { go } from '@/lib/router.svelte';
	import { session } from '@/lib/session.svelte';

	type Props = {
		token?: string;
		user: { email: string; name: string } | null;
	};

	const { token = '', user }: Props = $props();

	let password = $state('');
	let repeat = $state('');

	async function submit() {
		if (password !== repeat) {
			throw new Error('The passwords are not the same');
		}

		const resp = await setPasswordWithLink(token, password);
		session.login(resp.token, resp.user);
		go('/');
	}
</script>

<div class="card narrow">
	<h1>Set the password</h1>

	{#if !user}
		<p class="error">
			This link is invalid, used or expired. Ask an admin for a new one.
		</p>
	{:else}
		<p class="hint who">{user.name}, {user.email}</p>

		<Form onsubmit={submit} submit="Set and log in">
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
		</Form>
	{/if}
</div>

<style lang="scss">
	.who {
		margin-bottom: 1rem;
	}
</style>
