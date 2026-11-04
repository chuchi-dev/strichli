import { del, get, post, put } from './lib';

export type Session = {
	token: string;
	/** seconds since the unix epoch */
	timeout: number;
};

export type User = {
	id: string;
	email: string;
	name: string;
	admin: boolean;
	createdOn: string;
};

export type Authenticated = {
	session: Session;
	user: User;
};

export async function login(
	email: string,
	password: string,
): Promise<Authenticated> {
	return await post('users/login', { email, password });
}

export async function tokenAuth(): Promise<Authenticated> {
	return await post('users/tokenauth');
}

export async function logout(): Promise<void> {
	await post('users/logout');
}

export async function changePassword(
	password: string,
	newPassword: string,
): Promise<void> {
	await post('users/password', { password, newPassword });
}

export type UserWithSites = User & {
	/** the sites a user who is no admin sees */
	siteIds: string[];
};

export type UserReq = {
	email: string;
	name: string;
	/** required for a new user, else only to change it */
	password?: string;
	admin: boolean;
	siteIds: string[];
};

export async function listUsers(): Promise<UserWithSites[]> {
	return await get('users');
}

export async function createUser(req: UserReq): Promise<UserWithSites> {
	return await post('users', req);
}

export async function updateUser(
	id: string,
	req: UserReq,
): Promise<UserWithSites> {
	return await put(`users/${id}`, req);
}

export async function deleteUser(id: string): Promise<void> {
	await del(`users/${id}`);
}

export type ApiToken = {
	id: string;
	name: string;
	createdOn: string;
	lastUsed: string | null;
};

export async function listTokens(): Promise<ApiToken[]> {
	return await get('tokens');
}

export async function createToken(
	name: string,
): Promise<ApiToken & { secret: string }> {
	return await post('tokens', { name });
}

export async function deleteToken(id: string): Promise<void> {
	await del(`tokens/${id}`);
}
