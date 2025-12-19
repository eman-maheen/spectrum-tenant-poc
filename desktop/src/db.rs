use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        
        Ok(Database { conn })
    }
    
    pub fn init_schema(&self) -> Result<()> {
        let schema = include_str!("../schema.sql");
        self.conn.execute_batch(schema)?;
        Ok(())
    }
    
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

pub async fn init_db() -> Result<Database> {
    // Get app data directory
    let app_data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find app data directory"))?
        .join("spectrum-tenant-poc");
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&app_data_dir)?;
    
    let db_path = app_data_dir.join("spectrum.db");
    println!("Initializing database at: {:?}", db_path);
    
    let db = Database::new(db_path)?;
    db.init_schema()?;
    
    println!("Database initialized successfully");
    Ok(db)
}
