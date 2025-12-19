import { invoke } from '@tauri-apps/api/core';
import type { Client, Property } from '$lib/types';

export const tauriApi = {
  // Client operations
  async listClients(): Promise<Client[]> {
    const result = await invoke<string>('list_clients');
    return JSON.parse(result);
  },

  async getClient(id: string): Promise<Client | null> {
    const result = await invoke<string>('get_client', { id });
    return JSON.parse(result);
  },

  async createClient(name: string, email: string, phone?: string): Promise<string> {
    return await invoke<string>('create_client', { name, email, phone });
  },

  async updateClient(
    id: string,
    name?: string,
    email?: string,
    phone?: string
  ): Promise<void> {
    await invoke('update_client', { id, name, email, phone });
  },

  async deleteClient(id: string): Promise<void> {
    await invoke('delete_client', { id });
  },

  // Property operations
  async listProperties(): Promise<Property[]> {
    const result = await invoke<string>('list_properties');
    return JSON.parse(result);
  },

  async createProperty(
    clientId: string,
    name: string,
    address: string,
    city?: string,
    state?: string,
    zipCode?: string
  ): Promise<string> {
    return await invoke<string>('create_property', {
      clientId,
      name,
      address,
      city,
      stateVal: state,
      zipCode
    });
  },

  // Sync operations
  async syncPull(): Promise<string> {
    return await invoke<string>('sync_pull');
  },

  async checkConnection(): Promise<boolean> {
    return await invoke<boolean>('check_server_connection');
  }
};
