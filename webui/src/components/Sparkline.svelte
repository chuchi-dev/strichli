<script lang="ts">
	import type { Bucket } from '@/api/sites';
	import { day, num } from '@/lib/format';

	/** visitors per day, today last */
	const { days }: { days: Bucket[] } = $props();

	const WIDTH = 150;
	const HEIGHT = 36;

	const max = $derived(Math.max(1, ...days.map(d => d.visitors)));
	const band = $derived(days.length ? WIDTH / days.length : 0);
	const barW = $derived(Math.max(1, band - 1));
	const h = (v: number) => (v / max) * (HEIGHT - 2);
</script>

<svg
	width={WIDTH}
	height={HEIGHT}
	role="img"
	aria-label="Visitors of the last 30 days"
>
	<line x1="0" x2={WIDTH} y1={HEIGHT - 0.5} y2={HEIGHT - 0.5} class="base" />
	{#each days as d, i (d.date)}
		<rect
			x={band * i}
			y={HEIGHT - Math.max(h(d.visitors), d.visitors ? 1 : 0)}
			width={barW}
			height={Math.max(h(d.visitors), d.visitors ? 1 : 0)}
			rx="1"
			class:today={i === days.length - 1}
		>
			<title>{day(d.date)}: {num(d.visitors)} visitors</title>
		</rect>
	{/each}
</svg>

<style lang="scss">
	svg {
		display: block;
	}

	.base {
		stroke: var(--axis);
	}

	rect {
		fill: var(--bar-tint);

		&.today {
			fill: var(--series-1);
		}
	}
</style>
