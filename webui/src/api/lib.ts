import { getSession } from '@/lib/Session';

// in development the server runs separately (with cors enabled)
const API_ADDR = import.meta.env.DEV ? 'http://localhost:8080/api/' : '/api/';

export async function request(
	method: string,
	path: string,
	data: object | null = null,
): Promise<any> {
	const headers: Record<string, string> = {
		'content-type': 'application/json',
	};

	const token = getSession()?.token();
	if (token) headers['session-token'] = token;

	let resp: Response;
	try {
		resp = await fetch(API_ADDR + path, {
			method,
			headers,
			body: method === 'GET' ? undefined : JSON.stringify(data),
		});
	} catch (e: any) {
		throw new ApiError(e.message, { type: 'REQUEST', detail: e.message });
	}

	if (!resp.ok) {
		let err: ApiError;
		try {
			const obj = await resp.json();
			err = new ApiError(obj.message, obj.error);
		} catch (_e) {
			err = new ApiError(resp.statusText);
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

export type ErrorType =
	| {
			type:
				| 'LOGIN_INCORRECT'
				| 'INVALID_SESSION_TOKEN'
				| 'INVALID_API_TOKEN'
				| 'ADMIN_ONLY'
				| 'NOT_FOUND';
	  }
	| {
			type: 'LIMIT_REACHED';
			/** seconds to wait */
			detail: number;
	  }
	| {
			type: 'INVALID' | 'CONFLICT' | 'INTERNAL' | 'REQUEST';
			detail: string;
	  };

export class ApiError extends Error {
	inner: ErrorType;

	constructor(message: string, type: ErrorType | null = null) {
		super(message);

		this.inner = type ?? { type: 'INTERNAL', detail: message };
	}

	get type(): ErrorType['type'] {
		return this.inner.type;
	}
}
