import { writable } from 'svelte/store';
import type { ProcessNode, ConnectionEdge } from '$lib/simcraft/base';

export const selectedElement = writable<ProcessNode | ConnectionEdge | null>(null);
export const configPanelVisible = writable<boolean>(false);
