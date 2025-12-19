import { writable } from 'svelte/store';
import type { Client } from '$lib/types';

export const clients = writable<Client[]>([]);
export const selectedClient = writable<Client | null>(null);
export const isLoading = writable(false);
