import type { Request } from 'chuchi';
import type Session from './Session';
import type { Route, Router } from '.';

export default class LoadProps {
	router: Router;
	route: Route | null;
	req: Request;
	session: Session;
	redirect: string | null;

	constructor(obj: {
		router: Router;
		route: Route | null;
		req: Request;
		session: Session;
	}) {
		this.router = obj.router;
		this.route = obj.route;
		this.req = obj.req;
		this.session = obj.session;
		this.redirect = null;
	}

	setRedirect(url: string) {
		this.redirect = url;
	}
}

export type LoadPropsFn<
	P extends Record<string, any> = object,
	R extends Record<string, any> = object,
> = (props: P, lp: LoadProps) => Promise<R | null | void> | void;

export type ResolvedProps<T extends (...args: any) => any> = Awaited<
	ReturnType<T>
>;
