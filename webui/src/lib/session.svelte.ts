import { me, type User } from '@/api/users';

const STORAGE_KEY = 'STRICHLI_TOKEN';

class Session {
	token = $state<string | null>(localStorage.getItem(STORAGE_KEY));
	user = $state<User | null>(null);

	/** Loads the user of a stored token, a rejected one logs out. */
	async init() {
		if (!this.token) return;
		try {
			this.user = await me();
		} catch (e) {
			// only a rejected token logs out (see request), not a restart
			console.error('session check failed', e);
		}
	}

	login(token: string, user: User) {
		this.token = token;
		this.user = user;
		localStorage.setItem(STORAGE_KEY, token);
	}

	logout() {
		this.token = null;
		this.user = null;
		localStorage.removeItem(STORAGE_KEY);
	}
}

export const session = new Session();
