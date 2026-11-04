import { del, get, post, put } from './lib';

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

export type Stats = {
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
	/** the data fields of the keys in the range */
	fields: string[];
	rows: EventRow[];
};

/** days as `YYYY-MM-DD`, both included */
export type Range = { from: string; to: string };

const query = (r: Range) =>
	`from=${encodeURIComponent(r.from)}&to=${encodeURIComponent(r.to)}`;

export async function listSites(): Promise<SiteOverview[]> {
	return await get('sites');
}

export async function getSite(id: string): Promise<Site> {
	return await get(`sites/${id}`);
}

export async function createSite(domain: string, name: string): Promise<Site> {
	return await post('sites', { domain, name });
}

export async function updateSite(
	id: string,
	domain: string,
	name: string,
): Promise<Site> {
	return await put(`sites/${id}`, { domain, name });
}

export async function deleteSite(id: string): Promise<void> {
	await del(`sites/${id}`);
}

export async function getStats(id: string, range: Range): Promise<Stats> {
	return await get(`sites/${id}/stats?${query(range)}`);
}

/** the keys grouped by the values of the data fields (at most 5) */
export async function getEvents(
	id: string,
	range: Range,
	fields: string[],
): Promise<EventTable> {
	const f = encodeURIComponent(fields.join(','));
	return await get(`sites/${id}/events?${query(range)}&fields=${f}`);
}
