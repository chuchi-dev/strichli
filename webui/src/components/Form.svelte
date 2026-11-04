<script lang="ts">
	import type { Snippet } from 'svelte';
	import Button from './Button.svelte';

	// a form like genco's: it runs `onsubmit` once at a time, an error it
	// throws shows below the fields, with `done` the submit button says so
	// for a moment (e.g. "Saved"). `extra` buttons go after it.
	const {
		onsubmit,
		oncancel,
		submit = 'Save',
		done,
		extra,
		children,
	}: {
		onsubmit: () => unknown;
		oncancel?: () => void;
		submit?: string;
		done?: string;
		extra?: Snippet;
		children: Snippet;
	} = $props();

	let busy = $state(false);
	let error = $state<string | null>(null);
	let flash = $state(false);

	async function run(e: SubmitEvent) {
		e.preventDefault();
		if (busy) return;
		busy = true;
		error = null;
		try {
			await onsubmit();
			if (done) {
				flash = true;
				setTimeout(() => (flash = false), 1500);
			}
		} catch (err: any) {
			error = err?.message ?? String(err);
		} finally {
			busy = false;
		}
	}
</script>

<form class="form" onsubmit={run}>
	{@render children()}

	{#if error}
		<p class="error">{error}</p>
	{/if}

	<div class="row-buttons">
		<Button type="submit" kind="primary" disabled={busy}>
			{flash && done ? done : submit}
		</Button>
		{#if oncancel}
			<Button kind="secondary" onclick={oncancel}>Cancel</Button>
		{/if}
		{#if extra}
			{@render extra()}
		{/if}
	</div>
</form>
