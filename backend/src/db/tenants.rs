use sqlx::PgPool;
use anyhow::Result;
use uuid::Uuid;

pub struct Tenant {
    pub id: Uuid,
    pub schema_name: String,
    pub name: String,
}

pub async fn get_tenant_by_id(pool: &PgPool, tenant_id: Uuid) -> Result<Option<Tenant>> {
    let tenant = sqlx::query_as!(
        Tenant,
        r#"
        SELECT id, schema_name, name
        FROM public.tenants
        WHERE id = $1
        "#,
        tenant_id
    )
    .fetch_optional(pool)
    .await?;
    
    Ok(tenant)
}

pub async fn get_tenant_by_schema(pool: &PgPool, schema_name: &str) -> Result<Option<Tenant>> {
    let tenant = sqlx::query_as!(
        Tenant,
        r#"
        SELECT id, schema_name, name
        FROM public.tenants
        WHERE schema_name = $1
        "#,
        schema_name
    )
    .fetch_optional(pool)
    .await?;
    
    Ok(tenant)
}

pub async fn create_tenant(pool: &PgPool, name: &str, schema_name: &str) -> Result<Tenant> {
    let tenant = sqlx::query_as!(
        Tenant,
        r#"
        INSERT INTO public.tenants (schema_name, name)
        VALUES ($1, $2)
        RETURNING id, schema_name, name
        "#,
        schema_name,
        name
    )
    .fetch_one(pool)
    .await?;
    
    // Create the tenant schema
    sqlx::query("SELECT create_tenant_schema($1, $2)")
        .bind(tenant.id)
        .bind(&tenant.schema_name)
        .execute(pool)
        .await?;
    
    Ok(tenant)
}
