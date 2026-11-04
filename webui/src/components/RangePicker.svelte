<script lang="ts">
	import type { Range } from '@/api/sites';
	import { PRESETS } from '@/lib/range';

	type Props = {
		range: Range;
		/** the id of the preset, null for custom days */
		preset: string | null;
		onpreset: (id: string) => void;
		oncustom: (range: Range) => void;
	};

	const { range, preset, onpreset, oncustom }: Props = $props();

	// svelte-ignore state_referenced_locally
	let from = $state(range.from);
	// svelte-ignore state_referenced_locally
	let to = $state(range.to);

	function custom(e: SubmitEvent) {
		e.preventDefault();
		if (from && to && from <= to) oncustom({ from, to });
	}
</script>

<div class="picker">
	<div class="presets" role="group" aria-label="Time range">
		{#each PRESETS as p (p.id)}
			<button
				class:active={p.id === preset}
				aria-pressed={p.id === preset}
				onclick={() => onpreset(p.id)}
			>
				{p.label}
			</button>
		{/each}
	</div>

	<form class="custom" onsubmit={custom}>
		<input type="date" bind:value={from} aria-label="From" max={to} />
		<span class="muted">to</span>
		<input type="date" bind:value={to} aria-label="To" min={from} />
		<button type="submit" class:active={preset === null}>Show</button>
	</form>
</div>

<style lang="scss">
	.picker {
		display: flex;
		flex-wrap: wrap;
		gap: 0.75rem;
		align-items: center;
	}

	.presets {
		display: flex;
		flex-wrap: wrap;
		gap: 2px;
		padding: 2px;
		background-color: var(--wash);
		border: 1px solid var(--border);
		border-radius: 8px;
	}

	button {
		padding: 0.3rem 0.65rem;
		font-size: 0.85rem;
		color: var(--ink-2);
		background: none;
		border: none;
		border-radius: var(--radius);
		cursor: pointer;

		&:hover {
			background-color: var(--wash);
		}

		&.active {
			color: var(--ink);
			font-weight: 600;
			background-color: var(--surface);
			box-shadow: 0 0 0 1px var(--border);
		}
	}

	.custom {
		display: flex;
		gap: 0.4rem;
		align-items: center;

		input {
			padding: 0.25rem 0.4rem;
			font-size: 0.85rem;
		}

		button {
			border: 1px solid var(--border);
		}
	}
</style>
