import { Router as ChuchiRouter, Route as ChuchiRoute } from 'chuchi';
import type { Component as SvelteComponent } from 'svelte';
import { type LoadPropsFn } from './LoadProps';

export type Component = SvelteComponent<any, any, any>;
export type ComponentModule = {
	default: Component;
	/** pages need a login unless they are public */
	isPublic?: boolean;
	loadProps?: LoadPropsFn<any, any>;
};

export type Router = ChuchiRouter<ComponentModule>;

export type Route = ChuchiRoute<ComponentModule>;
