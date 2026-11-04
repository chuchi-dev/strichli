<script lang="ts">
	import type { Bucket } from '@/api/sites';
	import { bucketLabel, bucketTitle, num } from '@/lib/format';

	type Props = {
		buckets: Bucket[];
		/** the buckets are hours */
		hourly: boolean;
	};

	const { buckets, hourly }: Props = $props();

	const HEIGHT = 240;
	const TOP = 12;
	const BOTTOM = 26;
	const RIGHT = 8;
	const MAX_BAR = 24;

	let width = $state(0);
	let hover: number | null = $state(null);
	let showTable = $state(false);

	/** 1, 2 or 5 times a power of ten */
	function niceStep(raw: number): number {
		const pow = 10 ** Math.floor(Math.log10(raw));
		const f = raw / pow;
		return (f <= 1 ? 1 : f <= 2 ? 2 : f <= 5 ? 5 : 10) * pow;
	}

	const max = $derived(
		Math.max(0, ...buckets.map(b => Math.max(b.visitors, b.pageviews))),
	);
	const step = $derived(max <= 4 ? 1 : niceStep(max / 4));
	const yMax = $derived(Math.max(4, Math.ceil(max / step) * step));
	const ticks = $derived(
		Array.from({ length: Math.round(yMax / step) + 1 }, (_, i) => i * step),
	);

	const left = $derived(12 + 7 * num(yMax).length);
	const innerW = $derived(Math.max(0, width - left - RIGHT));
	const innerH = HEIGHT - TOP - BOTTOM;
	const band = $derived(buckets.length ? innerW / buckets.length : 0);
	// a 2px gap between columns while there is room for it
	const barW = $derived(
		Math.max(1, Math.min(MAX_BAR, band >= 6 ? band - 2 : band * 0.7)),
	);

	const y = (v: number) => TOP + innerH - (v / yMax) * innerH;
	const cx = (i: number) => left + band * i + band / 2;

	/** a column with a 4px rounded top, square at the baseline */
	function column(i: number, v: number): string {
		const x = cx(i) - barW / 2;
		const top = y(v);
		const h = TOP + innerH - top;
		const r = Math.min(4, barW / 2, h);

		return (
			`M${x},${TOP + innerH}V${top + r}Q${x},${top} ${x + r},${top}` +
			`H${x + barW - r}Q${x + barW},${top} ${x + barW},${top + r}` +
			`V${TOP + innerH}Z`
		);
	}

	const line = $derived(
		buckets
			.map((b, i) => `${i ? 'L' : 'M'}${cx(i)},${y(b.pageviews)}`)
			.join(''),
	);

	// labels at least ~60px apart, counted from the last bucket (today)
	const every = $derived(
		Math.max(1, Math.ceil(buckets.length / Math.max(1, innerW / 60))),
	);
	const showLabel = (i: number) => (buckets.length - 1 - i) % every === 0;

	const tooltipLeft = $derived(
		hover === null ? 0 : Math.min(Math.max(cx(hover), 80), width - 80),
	);

	function onKey(e: KeyboardEvent) {
		if (!buckets.length) return;
		if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') {
			e.preventDefault();
			const d = e.key === 'ArrowRight' ? 1 : -1;
			const start = hover ?? (d > 0 ? -1 : buckets.length);
			hover = Math.min(buckets.length - 1, Math.max(0, start + d));
		}
	}
</script>

<div class="chart">
	<div class="top">
		<ul class="legend">
			<li>
				<svg width="12" height="12" aria-hidden="true">
					<rect width="12" height="12" rx="2" class="visitors" />
				</svg>
				Visitors
			</li>
			<li>
				<svg width="16" height="12" aria-hidden="true">
					<path d="M1,6H15" class="pageviews" />
				</svg>
				Pageviews
			</li>
		</ul>

		<button class="toggle" onclick={() => (showTable = !showTable)}>
			{showTable ? 'Chart' : 'Table'}
		</button>
	</div>

	{#if showTable}
		<div class="table">
			<table class="list">
				<thead>
					<tr>
						<th>{hourly ? 'Hour' : 'Day'}</th>
						<th class="right">Visitors</th>
						<th class="right">Pageviews</th>
					</tr>
				</thead>
				<tbody>
					{#each buckets as b (b.date)}
						<tr>
							<td>{bucketTitle(b.date, hourly)}</td>
							<td class="right num">{num(b.visitors)}</td>
							<td class="right num">{num(b.pageviews)}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else}
		<div class="plot" bind:clientWidth={width}>
			{#if width}
				<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
				<svg
					{width}
					height={HEIGHT}
					role="img"
					aria-label="Visitors and pageviews, use the arrow keys for the values"
					tabindex="0"
					onkeydown={onKey}
					onpointerleave={() => (hover = null)}
					onblur={() => (hover = null)}
				>
					{#each ticks as t (t)}
						<line
							x1={left}
							x2={width - RIGHT}
							y1={y(t)}
							y2={y(t)}
							class={t === 0 ? 'baseline' : 'grid'}
						/>
						<text x={left - 8} y={y(t)} class="tick y">
							{num(t)}
						</text>
					{/each}

					{#if hover !== null}
						<rect
							x={left + band * hover}
							y={TOP}
							width={band}
							height={innerH}
							class="highlight"
						/>
					{/if}

					{#each buckets as b, i (b.date)}
						{#if b.visitors > 0}
							<path d={column(i, b.visitors)} class="visitors" />
						{/if}
						{#if showLabel(i)}
							<text x={cx(i)} y={HEIGHT - 8} class="tick x">
								{bucketLabel(b.date, hourly)}
							</text>
						{/if}
					{/each}

					<path d={line} class="pageviews" />

					{#if hover !== null}
						<circle
							cx={cx(hover)}
							cy={y(buckets[hover].pageviews)}
							r="4"
							class="marker"
						/>
					{/if}

					{#each buckets as b, i (b.date)}
						<rect
							role="presentation"
							x={left + band * i}
							y={TOP}
							width={band}
							height={innerH + BOTTOM}
							class="hit"
							onpointerenter={() => (hover = i)}
						/>
					{/each}
				</svg>

				{#if hover !== null}
					{@const b = buckets[hover]}
					<div class="tooltip" style:left="{tooltipLeft}px">
						<div class="title">{bucketTitle(b.date, hourly)}</div>
						<div class="row">
							<span class="key visitors"></span>
							<strong class="num">{num(b.visitors)}</strong>
							Visitors
						</div>
						<div class="row">
							<span class="key pageviews"></span>
							<strong class="num">{num(b.pageviews)}</strong>
							Pageviews
						</div>
					</div>
				{/if}
			{/if}
		</div>
	{/if}
</div>

<style lang="scss">
	.chart {
		position: relative;
	}

	.top {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.legend {
		display: flex;
		gap: 1rem;
		list-style: none;
		font-family: var(--mono);
		font-size: 0.78rem;
		color: var(--ink-2);

		li {
			display: flex;
			align-items: center;
			gap: 0.4rem;
		}
	}

	.toggle {
		padding: 0.2rem 0.5rem;
		font-family: var(--mono);
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		color: var(--ink-2);
		background: none;
		border: 2px solid var(--border);
		border-radius: var(--radius);
		cursor: pointer;

		&:hover {
			background-color: var(--wash);
		}
	}

	.plot {
		position: relative;
		height: 240px;

		svg {
			display: block;
			overflow: visible;

			&:focus-visible {
				outline: 2px solid var(--accent);
				outline-offset: 4px;
				border-radius: 4px;
			}
		}
	}

	.table {
		max-height: 240px;
		overflow-y: auto;
	}

	.grid {
		stroke: var(--grid);
		stroke-width: 1;
	}

	.baseline {
		stroke: var(--axis);
		stroke-width: 1;
	}

	.tick {
		font-family: var(--mono);
		font-size: 10.5px;
		fill: var(--muted);
		font-variant-numeric: tabular-nums;

		&.y {
			text-anchor: end;
			dominant-baseline: middle;
		}

		&.x {
			text-anchor: middle;
		}
	}

	rect.visitors,
	path.visitors {
		fill: var(--series-1);
	}

	path.pageviews {
		fill: none;
		stroke: var(--series-2);
		stroke-width: 2;
		stroke-linejoin: round;
		stroke-linecap: round;
	}

	.marker {
		fill: var(--series-2);
		stroke: var(--surface);
		stroke-width: 2;
	}

	.highlight {
		fill: var(--wash);
	}

	.hit {
		fill: transparent;
	}

	.tooltip {
		position: absolute;
		top: 0;
		transform: translateX(-50%);
		min-width: 9rem;
		padding: 0.5rem 0.65rem;
		font-size: 0.85rem;
		color: var(--ink-2);
		background-color: var(--surface);
		border: 2px solid var(--border);
		border-radius: var(--radius);
		box-shadow: 3px 3px 0 var(--shadow);
		pointer-events: none;

		.title {
			margin-bottom: 0.25rem;
			font-weight: 700;
			color: var(--ink);
		}

		.row {
			display: flex;
			align-items: center;
			gap: 0.4rem;
		}

		strong {
			color: var(--ink);
		}

		.key {
			display: inline-block;
			width: 12px;
			height: 2px;
			border-radius: 1px;

			&.visitors {
				background-color: var(--series-1);
			}

			&.pageviews {
				background-color: var(--series-2);
			}
		}
	}
</style>
