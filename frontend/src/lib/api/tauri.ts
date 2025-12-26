// frontend/src/lib/api/tauri.ts

import { invoke } from '@tauri-apps/api/core';
import type { Client, Property } from '$lib/types';

export const tauriApi = {
  // ========================================
  // CLIENT OPERATIONS
  // ========================================
  
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

  // ========================================
  // PROPERTY OPERATIONS
  // ========================================
  
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

  async deleteProperty(id: string): Promise<void> {
    await invoke('delete_property', { id });
  },

  // ========================================
  // SYNC OPERATIONS
  // ========================================
  
  /**
   * NEW: Bidirectional sync
   * - Pushes pending local changes to server
   * - Pulls latest data from server
   * - Handles conflicts (server wins)
   */
  async syncBidirectional(): Promise<string> {
    return await invoke<string>('sync_bidirectional');
  },

  async checkConnection(): Promise<boolean> {
    return await invoke<boolean>('check_server_connection');
  },

  async getPendingCount(): Promise<number> {
    return await invoke<number>('get_pending_count');
  }
};