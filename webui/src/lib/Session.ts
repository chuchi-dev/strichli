import { tokenAuth, type Authenticated, type User } from '@/api/users';
import { Writable } from 'chuchi/stores';

const STORAGE_KEY = 'SESSION_TOKEN';

export default class Session {
	/** the logged in user, null if not logged in */
	user: Writable<User | null>;
	private _token: string | null;

	constructor() {
		this.user = new Writable(null);
		this._token = null;
	}

	// will not throw
	static async init(): Promise<Session> {
		const me = new Session();
		/* @ts-ignore */
		globalThis.SESSION = me;

		const token = localStorage.getItem(STORAGE_KEY);
		if (!token) return me;

		me._token = token;
		try {
			// the session might get renewed
			me.setAuthed(await tokenAuth());
		} catch (e) {
			console.error('session invalid', e);
			me.invalidate();
		}

		return me;
	}

	token(): string | null {
		return this._token;
	}

	isLoggedIn(): boolean {
		return !!this.user.get();
	}

	setAuthed(auth: Authenticated) {
		this._token = auth.session.token;
		localStorage.setItem(STORAGE_KEY, auth.session.token);
		this.user.set(auth.user);
	}

	invalidate() {
		this._token = null;
		localStorage.removeItem(STORAGE_KEY);
		this.user.set(null);
	}
}

export function getSession(): Session | undefined {
	/* @ts-ignore */
	return globalThis.SESSION;
}
