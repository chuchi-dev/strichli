<script lang="ts">
	import type { SiteOverview } from '@/api/sites';
	import { day, num } from '@/lib/format';

	const { site }: { site: SiteOverview } = $props();

	const HEIGHT = 110;
	const PAD = 4;

	let width = $state(0);
	let hover: number | null = $state(null);

	const days = $derived(site.days);
	const visitors = $derived(days.reduce((a, d) => a + d.visitors, 0));
	const max = $derived(Math.max(4, ...days.map(d => d.pageviews)));
	const x = (i: number) =>
		days.length > 1 ? (i / (days.length - 1)) * width : width / 2;
	const y = (v: number) => PAD + (HEIGHT - 2 * PAD) * (1 - v / max);

	const line = (key: 'visitors' | 'pageviews') =>
		days.map((d, i) => `${i ? 'L' : 'M'}${x(i)},${y(d[key])}`).join('');
	const area = $derived(
		days.length
			? `${line('visitors')}L${x(days.length - 1)},${HEIGHT - PAD}L0,${HEIGHT - PAD}Z`
			: '',
	);

	function onMove(e: PointerEvent) {
		const rect = (e.currentTarget as SVGElement).getBoundingClientRect();
		const i = Math.round(
			((e.clientX - rect.left) / rect.width) * (days.length - 1),
		);
		hover = Math.min(days.length - 1, Math.max(0, i));
	}
</script>

<a href="/sites/{site.id}" class="card site-card">
	<div class="plot" bind:clientWidth={width}>
		{#if width}
			<svg
				{width}
				height={HEIGHT}
				role="img"
				aria-label="{site.name}: {num(
					visitors,
				)} visitors in the last 30 days"
				onpointermove={onMove}
				onpointerleave={() => (hover = null)}
			>
				{#each [0.25, 0.5, 0.75] as t (t)}
					<line
						x1="0"
						x2={width}
						y1={PAD + (HEIGHT - 2 * PAD) * t}
						y2={PAD + (HEIGHT - 2 * PAD) * t}
						class="grid"
					/>
				{/each}
				<line
					x1="0"
					x2={width}
					y1={HEIGHT - PAD}
					y2={HEIGHT - PAD}
					class="base"
				/>
				<path d={area} class="area" />
				<path d={line('pageviews')} class="pageviews" />
				<path d={line('visitors')} class="visitors" />
				{#if hover !== null}
					<line
						x1={x(hover)}
						x2={x(hover)}
						y1="0"
						y2={HEIGHT}
						class="cursor"
					/>
					<circle
						cx={x(hover)}
						cy={y(days[hover].visitors)}
						r="4"
						class="dot"
					/>
				{/if}
			</svg>
		{/if}
		{#if hover !== null}
			{@const d = days[hover]}
			<div class="readout">
				<strong>{day(d.date)}</strong>
				<span>{num(d.visitors)} visitors</span>
				<span class="muted">{num(d.pageviews)} pageviews</span>
			</div>
		{/if}
	</div>

	<div class="foot">
		<div class="names">
			<span class="name">{site.name}</span>
			{#if site.name !== site.domain}
				<span class="domain">{site.domain}</span>
			{/if}
		</div>
		<span class="count" title="Visitors in the last 30 days">
			{num(visitors)}
		</span>
	</div>
</a>

<style lang="scss">
	.site-card {
		display: block;
		padding: 1rem 1rem 0.9rem;
		color: inherit;
		text-decoration: none;
		transition:
			transform 0.1s ease,
			box-shadow 0.1s ease;

		&:hover {
			transform: translate(-2px, -2px);
			box-shadow: 6px 6px 0 var(--shadow);
		}

		&:focus-visible {
			outline: 3px solid var(--marker);
			outline-offset: 2px;
		}
	}

	.plot {
		position: relative;
		height: 110px;

		svg {
			display: block;
		}
	}

	.grid {
		stroke: var(--grid);
	}

	.base {
		stroke: var(--axis);
	}

	.area {
		fill: var(--bar-tint);
	}

	.visitors,
	.pageviews {
		fill: none;
		stroke-width: 2;
		stroke-linejoin: round;
	}

	.visitors {
		stroke: var(--series-1);
	}

	.pageviews {
		stroke: var(--series-2);
		stroke-dasharray: 4 3;
	}

	.cursor {
		stroke: var(--axis);
	}

	.dot {
		fill: var(--series-1);
		stroke: var(--surface);
		stroke-width: 2;
	}

	.readout {
		position: absolute;
		top: 0;
		left: 0;
		display: flex;
		gap: 0.5rem;
		padding: 0.15rem 0.4rem;
		font-family: var(--mono);
		font-size: 0.75rem;
		background-color: var(--surface);
		border: 2px solid var(--border);
		border-radius: var(--radius);
		pointer-events: none;
	}

	.foot {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		gap: 1rem;
		margin-top: 0.75rem;
	}

	.names {
		display: grid;
		min-width: 0;
	}

	.name {
		font-weight: 800;
		font-size: 1.2rem;
		letter-spacing: -0.01em;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.domain {
		font-family: var(--mono);
		font-size: 0.75rem;
		color: var(--muted);
	}

	.count {
		font-family: var(--mono);
		font-size: 1.7rem;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		line-height: 1;
	}
</style>
