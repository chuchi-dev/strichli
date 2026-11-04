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

export type ValueCount = { field: string; value: string; count: number };

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

export async function getValues(
	id: string,
	key: string,
	range: Range,
): Promise<ValueCount[]> {
	return await get(
		`sites/${id}/keys/${encodeURIComponent(key)}?${query(range)}`,
	);
}
