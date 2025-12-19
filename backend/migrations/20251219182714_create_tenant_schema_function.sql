-- Function to create a new tenant schema with all tables
CREATE OR REPLACE FUNCTION create_tenant_schema(p_tenant_id UUID, p_schema_name VARCHAR)
RETURNS void AS $$
BEGIN
    -- Create schema
    EXECUTE format('CREATE SCHEMA IF NOT EXISTS %I', p_schema_name);
    
    -- Create clients table (source of truth: CLIENT)
    EXECUTE format('
        CREATE TABLE %I.clients (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL,
            phone VARCHAR(50),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            version BIGINT NOT NULL DEFAULT 1,
            is_deleted BOOLEAN NOT NULL DEFAULT false
        )', p_schema_name);
    
    -- Create properties table (source of truth: CLIENT)
    EXECUTE format('
        CREATE TABLE %I.properties (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            client_id UUID NOT NULL REFERENCES %I.clients(id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            address TEXT NOT NULL,
            city VARCHAR(100),
            state VARCHAR(50),
            zip_code VARCHAR(20),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            version BIGINT NOT NULL DEFAULT 1,
            is_deleted BOOLEAN NOT NULL DEFAULT false
        )', p_schema_name, p_schema_name);
    
    -- Create buildings table (source of truth: CLIENT)
    EXECUTE format('
        CREATE TABLE %I.buildings (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            property_id UUID NOT NULL REFERENCES %I.properties(id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            floors INTEGER NOT NULL DEFAULT 1,
            year_built INTEGER,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            version BIGINT NOT NULL DEFAULT 1,
            is_deleted BOOLEAN NOT NULL DEFAULT false
        )', p_schema_name, p_schema_name);
    
    -- Create units table (source of truth: CLIENT)
    EXECUTE format('
        CREATE TABLE %I.units (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            building_id UUID NOT NULL REFERENCES %I.buildings(id) ON DELETE CASCADE,
            unit_number VARCHAR(50) NOT NULL,
            unit_type VARCHAR(50) NOT NULL,
            square_feet INTEGER,
            bedrooms INTEGER,
            bathrooms DECIMAL(3,1),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            version BIGINT NOT NULL DEFAULT 1,
            is_deleted BOOLEAN NOT NULL DEFAULT false
        )', p_schema_name, p_schema_name);
    
    -- Create reports table (source of truth: SERVER)
    EXECUTE format('
        CREATE TABLE %I.reports (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            property_id UUID NOT NULL REFERENCES %I.properties(id) ON DELETE CASCADE,
            report_type VARCHAR(100) NOT NULL,
            title VARCHAR(255) NOT NULL,
            pdf_url TEXT,
            status VARCHAR(50) NOT NULL DEFAULT ''pending'',
            generated_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            version BIGINT NOT NULL DEFAULT 1
        )', p_schema_name, p_schema_name);
    
    -- Create triggers for updated_at on all tables
    EXECUTE format('
        CREATE TRIGGER update_clients_updated_at
            BEFORE UPDATE ON %I.clients
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    ', p_schema_name);
    
    EXECUTE format('
        CREATE TRIGGER update_properties_updated_at
            BEFORE UPDATE ON %I.properties
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    ', p_schema_name);
    
    EXECUTE format('
        CREATE TRIGGER update_buildings_updated_at
            BEFORE UPDATE ON %I.buildings
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    ', p_schema_name);
    
    EXECUTE format('
        CREATE TRIGGER update_units_updated_at
            BEFORE UPDATE ON %I.units
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
    ', p_schema_name);
    
    -- Create indexes for foreign keys and common queries
    EXECUTE format('CREATE INDEX idx_%s_properties_client_id ON %I.properties(client_id)', 
        replace(p_schema_name, '-', '_'), p_schema_name);
    EXECUTE format('CREATE INDEX idx_%s_buildings_property_id ON %I.buildings(property_id)', 
        replace(p_schema_name, '-', '_'), p_schema_name);
    EXECUTE format('CREATE INDEX idx_%s_units_building_id ON %I.units(building_id)', 
        replace(p_schema_name, '-', '_'), p_schema_name);
    EXECUTE format('CREATE INDEX idx_%s_reports_property_id ON %I.reports(property_id)', 
        replace(p_schema_name, '-', '_'), p_schema_name);
    
    -- Grant permissions
    EXECUTE format('GRANT ALL PRIVILEGES ON SCHEMA %I TO spectum', p_schema_name);
    EXECUTE format('GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA %I TO spectum', p_schema_name);
    EXECUTE format('GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA %I TO spectum', p_schema_name);
END;
$$ LANGUAGE plpgsql;

-- Function to drop tenant schema
CREATE OR REPLACE FUNCTION drop_tenant_schema(p_schema_name VARCHAR)
RETURNS void AS $$
BEGIN
    EXECUTE format('DROP SCHEMA IF EXISTS %I CASCADE', p_schema_name);
END;
$$ LANGUAGE plpgsql;
