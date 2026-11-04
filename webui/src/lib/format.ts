const numbers = new Intl.NumberFormat('de-CH');

export function num(n: number): string {
	return numbers.format(n);
}

const MONTHS = [
	'Jan',
	'Feb',
	'Mar',
	'Apr',
	'May',
	'Jun',
	'Jul',
	'Aug',
	'Sep',
	'Oct',
	'Nov',
	'Dec',
];

/** `2026-11-04` to `4 Nov`, with the year `4 Nov 2026` */
export function day(date: string, withYear = false): string {
	const [y, m, d] = date.split('-');
	const s = `${Number(d)} ${MONTHS[Number(m) - 1]}`;
	return withYear ? `${s} ${y}` : s;
}

/** a bucket's date: a day or `2026-11-04 06:00` to `06:00` */
export function bucketLabel(date: string, hourly: boolean): string {
	return hourly ? date.slice(11) : day(date);
}

/** a bucket's date for the tooltip */
export function bucketTitle(date: string, hourly: boolean): string {
	if (!hourly) return day(date, true);
	return `${day(date.slice(0, 10), true)}, ${date.slice(11)}`;
}

/** an iso timestamp of the server to `4 Nov 2026` */
export function timestamp(ts: string): string {
	return day(ts.slice(0, 10), true);
}
