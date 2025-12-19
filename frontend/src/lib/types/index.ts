export interface Client {
  id: string;
  name: string;
  email: string;
  phone?: string;
  created_at: string;
  updated_at: string;
  version: number;
  sync_status?: string;
}

export interface Property {
  id: string;
  client_id: string;
  name: string;
  address: string;
  city?: string;
  state?: string;
  zip_code?: string;
  created_at: string;
  updated_at: string;
  version: number;
  sync_status?: string;
}

export interface Building {
  id: string;
  property_id: string;
  name: string;
  floors: number;
  year_built?: number;
  created_at: string;
  updated_at: string;
  version: number;
  sync_status?: string;
}

export interface Unit {
  id: string;
  building_id: string;
  unit_number: string;
  unit_type: string;
  square_feet?: number;
  bedrooms?: number;
  bathrooms?: number;
  created_at: string;
  updated_at: string;
  version: number;
  sync_status?: string;
}
