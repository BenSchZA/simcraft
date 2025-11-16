export * from './base';

import type { SimcraftAdapter } from './base';
import { BrowserAdapter } from './browser';
import { DesktopAdapter } from './desktop';

const isDesktopBuild = import.meta.env.VITE_BUILD_TARGET === 'desktop';

export const isDesktop = '__TAURI_INTERNALS__' in window || isDesktopBuild;
export const isBrowser = !isDesktop;

export const createAdapter: () => Promise<SimcraftAdapter | null> = async () => isDesktop ? new DesktopAdapter() : new BrowserAdapter();
