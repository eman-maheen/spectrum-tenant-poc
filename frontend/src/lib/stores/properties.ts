import { writable } from 'svelte/store';
import type { Property } from '$lib/types';

export const properties = writable<Property[]>([]);
export const selectedProperty = writable<Property | null>(null);
