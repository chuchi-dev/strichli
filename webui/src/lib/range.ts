import type { RangeQuery } from '@/api/sites';

/** the server counts them back from today in its timezone */
export const PRESETS = [
	{ id: 'today', label: 'Today' },
	{ id: '7d', label: '7 days' },
	{ id: '30d', label: '30 days' },
	{ id: '90d', label: '90 days' },
	{ id: '12m', label: '12 months' },
];

const DEFAULT_PRESET = '30d';

const ISO_DATE = /^\d{4}-\d{2}-\d{2}$/;

/**
 * The range of the page's search params: a preset (`?range=7d`) or custom
 * days (`?from=2026-10-01&to=2026-10-31`)
 */
export function rangeFromProps(props: {
	range?: string;
	from?: string;
	to?: string;
}): { query: RangeQuery; preset: string | null } {
	if (props.from && props.to) {
		if (ISO_DATE.test(props.from) && ISO_DATE.test(props.to)) {
			return { query: { from: props.from, to: props.to }, preset: null };
		}
	}

	const preset = PRESETS.some(p => p.id === props.range)
		? props.range!
		: DEFAULT_PRESET;
	return { query: { range: preset }, preset };
}
