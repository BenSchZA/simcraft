import { svelteTesting } from '@testing-library/svelte/vite';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { fileURLToPath } from "node:url";

export default defineConfig(() => {
	const isDesktopBuild = process.env.VITE_BUILD_TARGET === 'desktop';
	console.log('isDesktopBuild', isDesktopBuild);

	const filesToExclude = ["simcraft_web", "src/lib/simcraft/browser.ts", "src/lib/workers/simulation.worker.ts"];
	const pathsToExclude = filesToExclude.map((src) => {
		return fileURLToPath(new URL(src, import.meta.url));
	});
	console.log('pathsToExclude', pathsToExclude);

	return {
		plugins: [
			sveltekit(),
			tailwindcss(),
			wasm(),
			topLevelAwait()
			// ...(isDesktopBuild ? [] : [])
		],
		assetsInclude: ['**/*.wasm'],

		build: {
			rollupOptions: {
				external: [...pathsToExclude]
			}
		},

		server: {
			fs: {
				allow: [
					'../crates/simcraft_web/pkg',
				],
			},
		},

		esbuild: {
			supported: {
				'top-level-await': true
			}
		},

		test: {
			workspace: [
				{
					extends: './vite.config.ts',
					plugins: [svelteTesting()],

					test: {
						name: 'client',
						environment: 'jsdom',
						clearMocks: true,
						include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
						exclude: ['src/lib/server/**'],
						setupFiles: ['./vitest-setup-client.ts']
					}
				},
				{
					extends: './vite.config.ts',

					test: {
						name: 'server',
						environment: 'node',
						include: ['src/**/*.{test,spec}.{js,ts}'],
						exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
					}
				}
			]
		}
	};
});
