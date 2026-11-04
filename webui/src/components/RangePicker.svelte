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
		background-color: var(--surface);
		border: 2px solid var(--border);
		border-radius: var(--radius);
		box-shadow: 3px 3px 0 var(--shadow);
		overflow: hidden;
	}

	button {
		padding: 0.35rem 0.7rem;
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
	}

	.presets button + button {
		border-left: 2px solid var(--border);
	}

	.custom {
		display: flex;
		gap: 0.4rem;
		align-items: center;
		font-family: var(--mono);
		font-size: 0.8rem;

		input {
			padding: 0.25rem 0.4rem;
			font-size: 0.8rem;
		}

		button {
			border: 2px solid var(--border);
			border-radius: var(--radius);
		}
	}
</style>
