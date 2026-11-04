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
	// a chunky button that presses down into its shadow
	.button {
		display: inline-block;
		padding: 0.5rem 1rem;
		font-family: var(--mono);
		font-size: 0.8rem;
		font-weight: 600;
		letter-spacing: 0.05em;
		text-transform: uppercase;
		text-decoration: none;
		color: var(--ink);
		background-color: var(--surface);
		border: 2px solid var(--border);
		border-radius: var(--radius);
		box-shadow: 3px 3px 0 var(--shadow);
		cursor: pointer;
		transition:
			transform 0.08s ease,
			box-shadow 0.08s ease,
			background-color 0.15s ease;

		&:hover:not(:disabled) {
			transform: translate(-1px, -1px);
			box-shadow: 4px 4px 0 var(--shadow);
		}

		&:active:not(:disabled) {
			transform: translate(3px, 3px);
			box-shadow: 0 0 0 var(--shadow);
		}

		&:focus-visible {
			outline: 3px solid var(--marker);
			outline-offset: 2px;
		}

		&:disabled {
			opacity: 0.5;
			cursor: default;
		}
	}

	.primary {
		color: var(--button-ink);
		background-color: var(--button);
		border-color: var(--button-border);
	}

	.danger {
		color: var(--danger);
	}
</style>
