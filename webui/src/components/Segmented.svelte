<script lang="ts" generics="T extends string">
	// one of a few options in a row, the chosen one marked
	const {
		options,
		value,
		onchange,
		label,
	}: {
		options: { id: T; label: string }[];
		/** null when none is chosen */
		value: T | null;
		onchange: (id: T) => void;
		label: string;
	} = $props();
</script>

<div class="segmented" role="group" aria-label={label}>
	{#each options as o (o.id)}
		<button
			class:active={o.id === value}
			aria-pressed={o.id === value}
			onclick={() => onchange(o.id)}
		>
			{o.label}
		</button>
	{/each}
</div>

<style lang="scss">
	.segmented {
		display: inline-flex;
		flex-wrap: wrap;
		background-color: var(--surface);
		border: 2px solid var(--border);
		border-radius: var(--radius);
		box-shadow: 3px 3px 0 var(--shadow);
		overflow: hidden;
	}

	button {
		padding: 0.35rem 0.75rem;
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
</style>
