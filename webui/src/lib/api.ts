import { session } from './session.svelte';

export class ApiError extends Error {
	type: string;

	constructor(message: string, type: string) {
		super(message);
		this.type = type;
	}
}

/** in development vite passes /api on to the server */
export async function request(
	method: string,
	path: string,
	data: object | null = null,
): Promise<any> {
	const headers: Record<string, string> = {
		'content-type': 'application/json',
	};
	if (session.token) headers['session-token'] = session.token;

	let resp: Response;
	try {
		resp = await fetch('/api' + path, {
			method,
			headers,
			body: method === 'GET' ? undefined : JSON.stringify(data),
		});
	} catch (_e) {
		throw new ApiError('The server is not reachable.', 'REQUEST');
	}

	if (!resp.ok) {
		let err: ApiError;
		try {
			const obj = await resp.json();
			err = new ApiError(obj.message, obj.error.type);
		} catch (_e) {
			err = new ApiError(resp.statusText, 'INTERNAL');
		}

		// the session is gone, start over
		if (err.type === 'INVALID_SESSION_TOKEN') {
			session.logout();
			window.location.href = '/login';
		}

		throw err;
	}

	return await resp.json();
}

export const get = (path: string) => request('GET', path);
export const post = (path: string, data: object | null = null) =>
	request('POST', path, data);
export const put = (path: string, data: object) => request('PUT', path, data);
export const del = (path: string) => request('DELETE', path);

/**
 * For loads that run again (another column, another range): only the
 * answer of the newest load counts, a slow older one doesn't overwrite it.
 */
export function newest() {
	let seq = 0;
	return <T>(
		load: Promise<T>,
		then: (value: T) => void,
		onerror: (e: any) => void = () => {},
	) => {
		const n = ++seq;
		load.then(
			v => n === seq && then(v),
			e => n === seq && onerror(e),
		);
	};
}
