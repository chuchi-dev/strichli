import type { Range } from '@/api/sites';

export type Preset = {
	id: string;
	label: string;
	/** how many days back from today, 0 is only today */
	days: number;
};

export const PRESETS: Preset[] = [
	{ id: 'today', label: 'Today', days: 0 },
	{ id: '7d', label: '7 days', days: 6 },
	{ id: '30d', label: '30 days', days: 29 },
	{ id: '90d', label: '90 days', days: 89 },
	{ id: '12m', label: '12 months', days: 364 },
];

export const DEFAULT_PRESET = '30d';

/** `YYYY-MM-DD` in the browser's timezone */
export function isoDate(date: Date): string {
	const pad = (n: number) => String(n).padStart(2, '0');
	return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

export function presetRange(preset: Preset): Range {
	const to = new Date();
	const from = new Date();
	from.setDate(from.getDate() - preset.days);

	return { from: isoDate(from), to: isoDate(to) };
}

const ISO_DATE = /^\d{4}-\d{2}-\d{2}$/;

/**
 * The range of the page's search params: a preset (`?range=7d`) or custom
 * days (`?from=2026-10-01&to=2026-10-31`)
 */
export function rangeFromProps(props: {
	range?: string;
	from?: string;
	to?: string;
}): { range: Range; preset: string | null } {
	if (
		props.from &&
		props.to &&
		ISO_DATE.test(props.from) &&
		ISO_DATE.test(props.to)
	) {
		return { range: { from: props.from, to: props.to }, preset: null };
	}

	const preset =
		PRESETS.find(p => p.id === props.range) ??
		PRESETS.find(p => p.id === DEFAULT_PRESET)!;
	return { range: presetRange(preset), preset: preset.id };
}
