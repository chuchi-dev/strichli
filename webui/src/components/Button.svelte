<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		/** a link if set */
		href?: string;
		type?: 'button' | 'submit';
		kind?: 'primary' | 'secondary' | 'danger';
		disabled?: boolean;
		onclick?: (e: MouseEvent) => void;
		children: Snippet;
	};

	const {
		href,
		type = 'button',
		kind = 'primary',
		disabled = false,
		onclick,
		children,
	}: Props = $props();
</script>

{#if href}
	<a {href} class="button {kind}">{@render children()}</a>
{:else}
	<button {type} class="button {kind}" {disabled} {onclick}>
		{@render children()}
	</button>
{/if}

<style lang="scss">
	.button {
		display: inline-block;
		padding: 0.45rem 0.9rem;
		font-weight: 600;
		font-size: 0.9rem;
		text-decoration: none;
		border: 1px solid transparent;
		border-radius: var(--radius);
		cursor: pointer;
		transition:
			background-color 0.15s ease,
			border-color 0.15s ease;

		&:focus-visible {
			outline: 2px solid var(--accent);
			outline-offset: 2px;
		}

		&:disabled {
			opacity: 0.5;
			cursor: default;
		}
	}

	.primary {
		background-color: var(--accent);
		color: #fff;

		&:hover:not(:disabled) {
			background-color: var(--accent-ink);
		}
	}

	.secondary {
		background-color: var(--surface);
		border-color: var(--axis);
		color: var(--ink);

		&:hover:not(:disabled) {
			background-color: var(--wash);
		}
	}

	.danger {
		background-color: var(--surface);
		border-color: var(--danger);
		color: var(--danger);

		&:hover:not(:disabled) {
			background-color: var(--wash);
		}
	}
</style>
