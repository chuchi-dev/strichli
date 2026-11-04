<script lang="ts">
	import type { Snippet } from 'svelte';

	type Props = {
		/** a link if set */
		href?: string;
		type?: 'button' | 'submit';
		kind?: 'primary' | 'secondary' | 'danger';
		small?: boolean;
		disabled?: boolean;
		onclick?: (e: MouseEvent) => void;
		children: Snippet;
	};

	const {
		href,
		type = 'button',
		kind = 'primary',
		small = false,
		disabled = false,
		onclick,
		children,
	}: Props = $props();
</script>

{#if href}
	<a {href} class="button {kind}" class:small>{@render children()}</a>
{:else}
	<button {type} class="button {kind}" class:small {disabled} {onclick}>
		{@render children()}
	</button>
{/if}

<style lang="scss">
	// a chunky button that presses down into its shadow
	@use 'styles/mixins' as *;

	.button {
		@include caps(0.8rem);
		display: inline-block;
		padding: 0.5rem 1rem;
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

	.small {
		padding: 0.25rem 0.6rem;
		font-size: 0.72rem;
		box-shadow: 2px 2px 0 var(--shadow);
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
