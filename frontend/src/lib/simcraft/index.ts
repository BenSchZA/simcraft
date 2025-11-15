export * from './base';

import type { SimcraftAdapter } from './base';
import { DesktopAdapter } from './desktop';

export const isDesktop = '__TAURI_INTERNALS__' in window;
export const isBrowser = !isDesktop;

export const createAdapter: () => Promise<SimcraftAdapter | null> = async () => {
	if (isDesktop) {
		console.log('Creating desktop adapter');
		return new DesktopAdapter();
	} else {
		console.log('Creating browser adapter');
		const { BrowserAdapter } = await import('./browser');
		return new BrowserAdapter();
	}
};
