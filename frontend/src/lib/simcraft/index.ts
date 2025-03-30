export * from './base';

import type { SimcraftAdapter } from './base';
import { BrowserAdapter } from './browser';
import { DesktopAdapter } from './desktop';
import { RemoteAdapter } from './remote';

export const adapter: SimcraftAdapter =
	'__TAURI_INTERNALS__' in window ? new DesktopAdapter() : new BrowserAdapter();

export const isDesktop = adapter instanceof DesktopAdapter;
export const isBrowser = adapter instanceof BrowserAdapter;
