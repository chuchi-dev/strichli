<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getRouter, getSession } from '@/main';

	const { children }: { children: Snippet } = $props();

	const req = getRouter().currentRequest;
	const user = getSession().user;

	const path = $derived($req?.url.pathname ?? '/');
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

		{#if $user}
			<nav>
				<a href="/" class:active={active('/')}>Sites</a>
				{#if $user.admin}
					<a href="/users" class:active={active('/users')}>Users</a>
					<a href="/tokens" class:active={active('/tokens')}>
						API tokens
					</a>
				{/if}
				<a href="/me" class:active={active('/me')}>{$user.name}</a>
			</nav>
		{/if}
	</div>
</header>

<main>
	{@render children()}
</main>

<style lang="scss">
	header {
		background-color: var(--surface);
		border-bottom: 1px solid var(--border);
	}

	.inner,
	main {
		max-width: 1100px;
		margin: 0 auto;
		padding: 0 1.25rem;
	}

	.inner {
		display: flex;
		height: 3.5rem;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-weight: 700;
		font-size: 1.1rem;
		color: var(--ink);
		text-decoration: none;

		svg {
			width: 1.6rem;
			height: 1.6rem;
			stroke: var(--series-1);
		}

		.strike {
			stroke: var(--series-2);
		}
	}

	nav {
		display: flex;
		gap: 0.25rem;
		overflow-x: auto;

		a {
			padding: 0.35rem 0.6rem;
			border-radius: var(--radius);
			color: var(--ink-2);
			text-decoration: none;
			white-space: nowrap;

			&:hover {
				background-color: var(--wash);
			}

			&.active {
				color: var(--ink);
				font-weight: 600;
			}
		}
	}

	main {
		padding-top: 2rem;
		padding-bottom: 4rem;
	}

	@media (max-width: 500px) {
		.logo .name {
			display: none;
		}
	}
</style>
