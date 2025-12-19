-- Insert demo tenant
INSERT INTO public.tenants (id, schema_name, name)
VALUES (
    '00000000-0000-0000-0000-000000000001'::UUID,
    'tenant_demo',
    'Demo Tenant'
)
ON CONFLICT (schema_name) DO NOTHING;

-- Create schema for demo tenant
SELECT create_tenant_schema(
    '00000000-0000-0000-0000-000000000001'::UUID,
    'tenant_demo'
);

-- Insert sample data for testing
INSERT INTO tenant_demo.clients (id, name, email, phone)
VALUES 
    ('10000000-0000-0000-0000-000000000001'::UUID, 'John Doe', 'john@example.com', '555-0100'),
    ('10000000-0000-0000-0000-000000000002'::UUID, 'Jane Smith', 'jane@example.com', '555-0200')
ON CONFLICT (id) DO NOTHING;

INSERT INTO tenant_demo.properties (id, client_id, name, address, city, state, zip_code)
VALUES 
    (
        '20000000-0000-0000-0000-000000000001'::UUID,
        '10000000-0000-0000-0000-000000000001'::UUID,
        'Sunset Apartments',
        '123 Main Street',
        'Springfield',
        'IL',
        '62701'
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO tenant_demo.buildings (id, property_id, name, floors, year_built)
VALUES 
    (
        '30000000-0000-0000-0000-000000000001'::UUID,
        '20000000-0000-0000-0000-000000000001'::UUID,
        'Building A',
        5,
        2015
    )
ON CONFLICT (id) DO NOTHING;

INSERT INTO tenant_demo.units (id, building_id, unit_number, unit_type, square_feet, bedrooms, bathrooms)
VALUES 
    (
        '40000000-0000-0000-0000-000000000001'::UUID,
        '30000000-0000-0000-0000-000000000001'::UUID,
        '101',
        '1BR',
        750,
        1,
        1.0
    ),
    (
        '40000000-0000-0000-0000-000000000002'::UUID,
        '30000000-0000-0000-0000-000000000001'::UUID,
        '102',
        '2BR',
        1100,
        2,
        2.0
    )
ON CONFLICT (id) DO NOTHING;