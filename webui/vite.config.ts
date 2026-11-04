import { resolve } from 'path';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// https://vitejs.dev/config/
export default defineConfig(() => {
	return {
		plugins: [svelte()],
		resolve: {
			alias: [{ find: '@', replacement: resolve(__dirname, 'src') }],
		},
		server: {
			// the server, `cargo run` in server/
			proxy: { '/api': 'http://127.0.0.1:8080' },
		},
	};
});
