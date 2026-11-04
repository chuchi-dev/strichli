import { tokenAuth, type Authenticated, type User } from '@/api/users';

const STORAGE_KEY = 'STRICHLI_TOKEN';

class Session {
	token = $state<string | null>(localStorage.getItem(STORAGE_KEY));
	user = $state<User | null>(null);

	/** Loads the user of a stored token, the session might get renewed. */
	async init() {
		if (!this.token) return;
		try {
			this.login(await tokenAuth());
		} catch (e) {
			// a rejected token logs out (see request)
			console.error('session check failed', e);
		}
	}

	login(auth: Authenticated) {
		this.token = auth.session.token;
		this.user = auth.user;
		localStorage.setItem(STORAGE_KEY, auth.session.token);
	}

	logout() {
		this.token = null;
		this.user = null;
		localStorage.removeItem(STORAGE_KEY);
	}
}

export const session = new Session();
