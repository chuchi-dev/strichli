<script lang="ts">
	import Button from '@/components/Button.svelte';

	const { error }: { error: any } = $props();
</script>

<h1>Error</h1>

<p class="error">{error?.message ?? error}</p>

<div class="btn">
	<Button href="/">All sites</Button>
</div>

<style lang="scss">
	.btn {
		margin-top: 1.5rem;
	}
</style>
