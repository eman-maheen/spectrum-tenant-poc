use anyhow::Result;
use rusqlite::{Connection, params};
use serde_json::Value;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Database {
    pub conn: Connection,  // Changed to pub
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        Ok(Database { conn })
    }
    
    pub fn init_schema(&self) -> Result<()> {
        let schema = include_str!("../schema.sql");
        self.conn.execute_batch(schema)?;
        Ok(())
    }
    
    // Client operations
    pub fn list_clients(&self) -> Result<Vec<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, email, phone, created_at, updated_at, version, _sync_status 
             FROM clients WHERE is_deleted = 0 ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "email": row.get::<_, String>(2)?,
                "phone": row.get::<_, Option<String>>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "updated_at": row.get::<_, String>(5)?,
                "version": row.get::<_, i64>(6)?,
                "sync_status": row.get::<_, String>(7)?,
            }))
        })?;
        
        let mut clients = Vec::new();
        for row in rows {
            clients.push(row?);
        }
        Ok(clients)
    }
    
    pub fn get_client(&self, id: &str) -> Result<Option<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, email, phone, created_at, updated_at, version, _sync_status 
             FROM clients WHERE id = ?1 AND is_deleted = 0"
        )?;
        
        let result = stmt.query_row([id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "email": row.get::<_, String>(2)?,
                "phone": row.get::<_, Option<String>>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "updated_at": row.get::<_, String>(5)?,
                "version": row.get::<_, i64>(6)?,
                "sync_status": row.get::<_, String>(7)?,
            }))
        });
        
        match result {
            Ok(client) => Ok(Some(client)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    pub fn create_client(&self, name: &str, email: &str, phone: Option<&str>) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO clients (id, name, email, phone, created_at, updated_at, _sync_status) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending')",
            params![&id, name, email, phone, &now, &now],
        )?;
        
        Ok(id)
    }
    
    pub fn update_client(&self, id: &str, name: Option<&str>, email: Option<&str>, phone: Option<&str>) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "UPDATE clients 
             SET name = COALESCE(?1, name),
                 email = COALESCE(?2, email),
                 phone = COALESCE(?3, phone),
                 updated_at = ?4,
                 version = version + 1,
                 _sync_status = 'pending'
             WHERE id = ?5",
            params![name, email, phone, &now, id],
        )?;
        
        Ok(())
    }
    
    pub fn delete_client(&self, id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "UPDATE clients 
             SET is_deleted = 1,
                 updated_at = ?1,
                 version = version + 1,
                 _sync_status = 'pending'
             WHERE id = ?2",
            params![&now, id],
        )?;
        
        Ok(())
    }
    
    // Property operations
    pub fn list_properties(&self) -> Result<Vec<Value>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, client_id, name, address, city, state, zip_code, 
                    created_at, updated_at, version, _sync_status 
             FROM properties WHERE is_deleted = 0 ORDER BY created_at DESC"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "client_id": row.get::<_, String>(1)?,
                "name": row.get::<_, String>(2)?,
                "address": row.get::<_, String>(3)?,
                "city": row.get::<_, Option<String>>(4)?,
                "state": row.get::<_, Option<String>>(5)?,
                "zip_code": row.get::<_, Option<String>>(6)?,
                "created_at": row.get::<_, String>(7)?,
                "updated_at": row.get::<_, String>(8)?,
                "version": row.get::<_, i64>(9)?,
                "sync_status": row.get::<_, String>(10)?,
            }))
        })?;
        
        let mut properties = Vec::new();
        for row in rows {
            properties.push(row?);
        }
        Ok(properties)
    }
    
    pub fn create_property(
        &self,
        client_id: &str,
        name: &str,
        address: &str,
        city: Option<&str>,
        state: Option<&str>,
        zip_code: Option<&str>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO properties (id, client_id, name, address, city, state, zip_code, created_at, updated_at, _sync_status) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'pending')",
            params![&id, client_id, name, address, city, state, zip_code, &now, &now],
        )?;
        
        Ok(id)
    }
}

pub async fn init_db() -> Result<Database> {
    let app_data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find app data directory"))?
        .join("spectrum-tenant-poc");
    
    std::fs::create_dir_all(&app_data_dir)?;
    
    let db_path = app_data_dir.join("spectrum.db");
    println!("Initializing database at: {:?}", db_path);
    
    let db = Database::new(db_path)?;
    db.init_schema()?;
    
    println!("Database initialized successfully");
    Ok(db)
}
