import { del, get, post, put } from '@/lib/api';

export type Site = {
	id: string;
	domain: string;
	name: string;
	createdOn: string;
	/** the script tag for the site's pages */
	snippet: string;
};

export type Bucket = {
	/** `YYYY-MM-DD` or `YYYY-MM-DD HH:00` */
	date: string;
	visitors: number;
	pageviews: number;
};

export type SiteOverview = Site & {
	/** the last 30 days, today last */
	days: Bucket[];
};

/** days as `YYYY-MM-DD`, both included */
export type Range = { from: string; to: string };

/** a preset the server counts back from today, or days */
export type RangeQuery = { range: string } | Range;

export type Stats = {
	/** the days of the stats */
	range: Range;
	visitors: number;
	pageviews: number;
	/** the buckets are hours */
	hourly: boolean;
	buckets: Bucket[];
	pages: { path: string; pageviews: number }[];
	/** referrer null is a direct visit */
	referrers: { referrer: string | null; visitors: number }[];
	keys: { key: string; count: number }[];
};

export type EventRow = {
	key: string;
	/** field to value, null if the event had no such field */
	values: Record<string, string | number | boolean | null>;
	count: number;
};

export type EventTable = {
	range: Range;
	/** the data fields of the keys in the range */
	fields: string[];
	rows: EventRow[];
};

const query = (q: RangeQuery) =>
	new URLSearchParams(q as Record<string, string>).toString();

export async function listSites(): Promise<SiteOverview[]> {
	return await get('/sites');
}

export async function getSite(id: string): Promise<Site> {
	return await get(`/sites/${id}`);
}

export async function createSite(domain: string, name: string): Promise<Site> {
	return await post('/sites', { domain, name });
}

export async function updateSite(
	id: string,
	domain: string,
	name: string,
): Promise<Site> {
	return await put(`/sites/${id}`, { domain, name });
}

export async function deleteSite(id: string): Promise<void> {
	await del(`/sites/${id}`);
}

export async function getStats(id: string, q: RangeQuery): Promise<Stats> {
	return await get(`/sites/${id}/stats?${query(q)}`);
}

/** the keys grouped by the values of the data fields (at most 5) */
export async function getEvents(
	id: string,
	q: RangeQuery,
	fields: string[],
): Promise<EventTable> {
	const f = encodeURIComponent(fields.join(','));
	return await get(`/sites/${id}/events?${query(q)}&fields=${f}`);
}
