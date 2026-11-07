import { del, get, post, put } from '@/lib/api';

export type User = {
	id: string;
	email: string;
	name: string;
	admin: boolean;
	createdOn: string;
};

/** a new session */
export type Login = {
	token: string;
	user: User;
};

export async function login(email: string, password: string): Promise<Login> {
	return await post('/users/login', { email, password });
}

export async function me(): Promise<User> {
	return await get('/users/me');
}

export async function logout(): Promise<void> {
	await post('/users/logout');
}

export async function changePassword(
	password: string,
	newPassword: string,
): Promise<void> {
	await post('/users/password', { password, newPassword });
}

export async function passwordLinkUser(
	token: string,
): Promise<{ email: string; name: string }> {
	return await get(`/users/password-link?token=${encodeURIComponent(token)}`);
}

export async function setPasswordWithLink(
	token: string,
	password: string,
): Promise<Login> {
	return await post('/users/password-link', { token, password });
}

export type UserWithSites = User & {
	/** the sites a user who is no admin sees */
	siteIds: string[];
	/** for a new user without a password: where they set it */
	passwordLink?: string;
};

export type UserReq = {
	email: string;
	name: string;
	/** a new user without one sets it with a link, else only to change it */
	password?: string;
	admin: boolean;
	siteIds: string[];
};

export async function listUsers(): Promise<UserWithSites[]> {
	return await get('/users');
}

export async function createUser(req: UserReq): Promise<UserWithSites> {
	return await post('/users', req);
}

export async function updateUser(
	id: string,
	req: UserReq,
): Promise<UserWithSites> {
	return await put(`/users/${id}`, req);
}

export async function deleteUser(id: string): Promise<void> {
	await del(`/users/${id}`);
}

export type ApiToken = {
	id: string;
	name: string;
	/** adds and changes sites, else it only reads */
	canWrite: boolean;
	createdOn: string;
	lastUsed: string | null;
};

export async function listTokens(): Promise<ApiToken[]> {
	return await get('/tokens');
}

/** `write` only for admins */
export async function createToken(
	name: string,
	write: boolean,
): Promise<ApiToken & { secret: string }> {
	return await post('/tokens', { name, write });
}

export async function deleteToken(id: string): Promise<void> {
	await del(`/tokens/${id}`);
}
