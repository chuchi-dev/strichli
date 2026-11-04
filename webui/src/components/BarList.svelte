<script lang="ts" generics="T">
	import type { Snippet } from 'svelte';
	import { num } from '@/lib/format';

	type Props = {
		title: string;
		/** the column header of the value */
		unit: string;
		items: T[];
		value: (item: T) => number;
		label: Snippet<[T]>;
		empty?: string;
	};

	const {
		title,
		unit,
		items,
		value,
		label,
		empty = 'Nothing yet',
	}: Props = $props();

	const LIMIT = 10;

	let expanded = $state(false);

	const max = $derived(Math.max(1, ...items.map(value)));
	const shown = $derived(expanded ? items : items.slice(0, LIMIT));
</script>

<div class="card bar-list">
	<div class="head">
		<h2>{title}</h2>
		<span class="unit">{unit}</span>
	</div>

	{#if !items.length}
		<p class="muted">{empty}</p>
	{:else}
		<ul>
			{#each shown as item, i (i)}
				<li>
					<span
						class="bar"
						style:width="{(value(item) / max) * 100}%"
					></span>
					<span class="name">{@render label(item)}</span>
					<span class="value num">{num(value(item))}</span>
				</li>
			{/each}
		</ul>

		{#if items.length > LIMIT}
			<button class="more" onclick={() => (expanded = !expanded)}>
				{expanded ? 'Show less' : `Show all ${items.length}`}
			</button>
		{/if}
	{/if}
</div>

<style lang="scss">
	.head {
		display: flex;
		justify-content: space-between;
		align-items: baseline;

		.unit {
			font-size: 0.85rem;
			color: var(--muted);
		}
	}

	ul {
		display: grid;
		gap: 2px;
		list-style: none;
	}

	li {
		position: relative;
		display: flex;
		justify-content: space-between;
		gap: 1rem;
		padding: 0.3rem 0.5rem;
		font-size: 0.9rem;
	}

	.bar {
		position: absolute;
		inset: 0 auto 0 0;
		background-color: var(--bar-tint);
		border-radius: 0 4px 4px 0;
	}

	.name,
	.value {
		position: relative;
	}

	.name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.more {
		margin-top: 0.5rem;
		font-size: 0.85rem;
		color: var(--accent-ink);
		background: none;
		border: none;
		cursor: pointer;
	}
</style>
