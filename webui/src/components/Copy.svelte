<script lang="ts">
	import Button from './Button.svelte';

	const { text }: { text: string } = $props();

	let copied = $state(false);

	async function copy() {
		await navigator.clipboard.writeText(text);
		copied = true;
		setTimeout(() => (copied = false), 1500);
	}
</script>

<div class="copy">
	<pre class="code">{text}</pre>
	<Button kind="secondary" onclick={copy}>
		{copied ? 'Copied' : 'Copy'}
	</Button>
</div>

<style lang="scss">
	.copy {
		display: flex;
		gap: 0.5rem;
		align-items: flex-start;

		pre {
			flex: 1;
		}
	}
</style>
