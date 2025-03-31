export * from './base';

import type { SimcraftAdapter } from './base';
import { BrowserAdapter } from './browser';
import { DesktopAdapter } from './desktop';
import { RemoteAdapter } from './remote';

export const isDesktop = '__TAURI_INTERNALS__' in window;
export const isBrowser = !isDesktop;

export const createAdapter: () => SimcraftAdapter = () =>
	isDesktop ? new DesktopAdapter() : new BrowserAdapter();
