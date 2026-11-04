<script lang="ts">
	import type { Snippet } from 'svelte';
	import { current } from '@/lib/router.svelte';
	import { session } from '@/lib/session.svelte';

	const { children }: { children: Snippet } = $props();

	const path = $derived(current.path);
	const user = $derived(session.user);
	const active = (p: string) =>
		p === '/' ? path === '/' || path.startsWith('/sites/') : path === p;
</script>

<header>
	<div class="inner">
		<a href="/" class="logo" aria-label="strichli, all sites">
			<svg viewBox="0 0 32 32" aria-hidden="true">
				<g stroke-width="3" stroke-linecap="round">
					<path d="M6 6v20M12 6v20M18 6v20M24 6v20" />
					<path d="M3 22 29 9" class="strike" />
				</g>
			</svg>
			<span class="name">strichli</span>
		</a>

		{#if user}
			<nav>
				<a href="/" class:active={active('/')}>Sites</a>
				{#if user.admin}
					<a href="/users" class:active={active('/users')}>Users</a>
					<a href="/tokens" class:active={active('/tokens')}>
						API tokens
					</a>
				{/if}
				<a href="/me" class:active={active('/me')}>{user.name}</a>
			</nav>
		{/if}
	</div>
</header>

<main>
	{@render children()}
</main>

<style lang="scss">
	@use 'styles/mixins' as *;

	header {
		background-color: var(--surface);
		border-bottom: 2px solid var(--border);
	}

	.inner,
	main {
		max-width: 1160px;
		margin: 0 auto;
		padding: 0 1.25rem;
	}

	.inner {
		display: flex;
		height: 4rem;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 0.55rem;
		font-weight: 800;
		font-size: 1.45rem;
		letter-spacing: -0.02em;
		color: var(--ink);
		text-decoration: none;

		svg {
			width: 2rem;
			height: 2rem;
			stroke: var(--series-1);
		}

		.strike {
			stroke: var(--marker);
		}
	}

	nav {
		display: flex;
		gap: 0.25rem;
		overflow-x: auto;

		a {
			padding: 0.35rem 0.6rem;
			@include caps(0.8rem);
			color: var(--ink-2);
			text-decoration: none;
			white-space: nowrap;
			border-radius: var(--radius);

			&:hover {
				color: var(--ink);
				background-color: var(--wash);
			}

			// a marker stroke under the current page
			&.active {
				color: var(--ink);
				background-image: linear-gradient(var(--marker), var(--marker));
				background-size: calc(100% - 1.2rem) 4px;
				background-position: center bottom 0.15rem;
				background-repeat: no-repeat;
			}
		}
	}

	main {
		padding-top: 2.5rem;
		padding-bottom: 4rem;
	}

	@media (max-width: 500px) {
		.logo .name {
			display: none;
		}
	}
</style>
