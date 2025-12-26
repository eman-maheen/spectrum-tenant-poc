// desktop/src/sync.rs - COMPLETE BIDIRECTIONAL SYNC

use anyhow::Result;
use serde::{Deserialize, Serialize};
use rusqlite::Connection;

const API_BASE_URL: &str = "http://localhost:3000/api";
const TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingClient {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PendingProperty {
    pub id: String,
    pub client_id: String,
    pub name: String,
    pub address: String,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
}

pub struct SyncService {
    pub api_url: String,
    pub tenant_id: String,
    pub client: reqwest::Client,
}

impl SyncService {
    pub fn new() -> Self {
        Self {
            api_url: API_BASE_URL.to_string(),
            tenant_id: TENANT_ID.to_string(),
            client: reqwest::Client::new(),
        }
    }

    // ========================================
    // BIDIRECTIONAL SYNC - THE MAIN FUNCTION
    // ========================================
    pub async fn sync_bidirectional(&self, conn: &Connection) -> Result<String> {
        let mut stats = SyncStats::default();
        
        // STEP 1: Push pending creates/updates/deletes to server
        println!("📤 Pushing pending changes to server...");
        self.push_pending_clients(conn, &mut stats).await?;
        self.push_pending_properties(conn, &mut stats).await?;
        self.push_pending_deletes(conn, &mut stats).await?;
        
        // STEP 2: Pull latest from server (including soft deletes)
        println!("📥 Pulling latest data from server...");
        self.pull_and_merge_clients(conn, &mut stats).await?;
        self.pull_and_merge_properties(conn, &mut stats).await?;
        
        Ok(stats.summary())
    }

    // ========================================
    // PUSH PENDING CHANGES
    // ========================================
    
    async fn push_pending_clients(&self, conn: &Connection, stats: &mut SyncStats) -> Result<()> {
        // Collect all pending clients first (no await while holding conn)
        let mut stmt = conn.prepare(
            "SELECT id, name, email, phone FROM clients 
             WHERE _sync_status = 'pending' AND is_deleted = 0"
        )?;
        
        let clients: Vec<PendingClient> = stmt.query_map([], |row| {
            Ok(PendingClient {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                phone: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        // Now do async operations and collect results
        for client in clients {
            match self.push_client(&client.name, &client.email, client.phone.as_deref()).await {
                Ok(server_data) => {
                    let server_id = server_data["id"].as_str().unwrap();
                    
                    // Replace local UUID with server ID
                    conn.execute(
                        "UPDATE clients SET 
                         id = ?1, 
                         version = ?2,
                         created_at = ?3,
                         updated_at = ?4,
                         _sync_status = 'synced'
                         WHERE id = ?5",
                        rusqlite::params![
                            server_id,
                            server_data["version"].as_i64().unwrap_or(1),
                            server_data["created_at"].as_str().unwrap_or(""),
                            server_data["updated_at"].as_str().unwrap_or(""),
                            &client.id
                        ],
                    )?;
                    
                    stats.clients_pushed += 1;
                    println!("✅ Pushed client: {} -> {}", client.name, server_id);
                }
                Err(e) => {
                    eprintln!("❌ Failed to push client {}: {}", client.name, e);
                    stats.errors += 1;
                }
            }
        }
        
        Ok(())
    }
    
    async fn push_pending_properties(&self, conn: &Connection, stats: &mut SyncStats) -> Result<()> {
        // Collect properties first
        let mut stmt = conn.prepare(
            "SELECT id, client_id, name, address, city, state, zip_code 
             FROM properties 
             WHERE _sync_status = 'pending' AND is_deleted = 0"
        )?;
        
        let properties: Vec<PendingProperty> = stmt.query_map([], |row| {
            Ok(PendingProperty {
                id: row.get(0)?,
                client_id: row.get(1)?,
                name: row.get(2)?,
                address: row.get(3)?,
                city: row.get(4)?,
                state: row.get(5)?,
                zip_code: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        // Now do async operations
        for prop in properties {
            match self.push_property(
                &prop.client_id,
                &prop.name,
                &prop.address,
                prop.city.as_deref(),
                prop.state.as_deref(),
                prop.zip_code.as_deref(),
            ).await {
                Ok(server_data) => {
                    let server_id = server_data["id"].as_str().unwrap();
                    
                    conn.execute(
                        "UPDATE properties SET 
                         id = ?1,
                         version = ?2,
                         created_at = ?3,
                         updated_at = ?4,
                         _sync_status = 'synced'
                         WHERE id = ?5",
                        rusqlite::params![
                            server_id,
                            server_data["version"].as_i64().unwrap_or(1),
                            server_data["created_at"].as_str().unwrap_or(""),
                            server_data["updated_at"].as_str().unwrap_or(""),
                            &prop.id
                        ],
                    )?;
                    
                    stats.properties_pushed += 1;
                    println!("✅ Pushed property: {} -> {}", prop.name, server_id);
                }
                Err(e) => {
                    eprintln!("❌ Failed to push property {}: {}", prop.name, e);
                    stats.errors += 1;
                }
            }
        }
        
        Ok(())
    }
    
    async fn push_pending_deletes(&self, conn: &Connection, stats: &mut SyncStats) -> Result<()> {
        // Collect IDs first
        let mut stmt = conn.prepare(
            "SELECT id FROM clients WHERE is_deleted = 1 AND _sync_status != 'synced'"
        )?;
        
        let client_ids: Vec<String> = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        
        drop(stmt); // Explicitly drop to release borrow
        
        // Delete on server
        for id in client_ids {
            if let Err(e) = self.delete_client(&id).await {
                eprintln!("❌ Failed to delete client {} on server: {}", id, e);
                stats.errors += 1;
            } else {
                conn.execute(
                    "UPDATE clients SET _sync_status = 'synced' WHERE id = ?1",
                    [&id],
                )?;
                stats.clients_deleted += 1;
            }
        }
        
        // Same for properties
        let mut stmt = conn.prepare(
            "SELECT id FROM properties WHERE is_deleted = 1 AND _sync_status != 'synced'"
        )?;
        
        let prop_ids: Vec<String> = stmt.query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        
        drop(stmt); // Explicitly drop
        
        for id in prop_ids {
            if let Err(e) = self.delete_property(&id).await {
                eprintln!("❌ Failed to delete property {} on server: {}", id, e);
                stats.errors += 1;
            } else {
                conn.execute(
                    "UPDATE properties SET _sync_status = 'synced' WHERE id = ?1",
                    [&id],
                )?;
                stats.properties_deleted += 1;
            }
        }
        
        Ok(())
    }

    // ========================================
    // PULL AND MERGE SERVER DATA
    // ========================================
    
    async fn pull_and_merge_clients(&self, conn: &Connection, stats: &mut SyncStats) -> Result<()> {
        let clients = self.pull_clients().await?;
        
        for client in clients {
            let id = client["id"].as_str().unwrap();
            let version = client["version"].as_i64().unwrap_or(1);
            let is_deleted = client["is_deleted"].as_bool().unwrap_or(false);
            
            // Check if we have this locally
            let local_version: Option<i64> = conn
                .query_row(
                    "SELECT version FROM clients WHERE id = ?1",
                    [id],
                    |row| row.get(0),
                )
                .ok();
            
            // Conflict resolution: Server wins if version is newer OR if we don't have it
            if local_version.is_none() || version > local_version.unwrap() {
                if is_deleted {
                    // Server says it's deleted - mark as deleted locally
                    conn.execute(
                        "UPDATE clients SET is_deleted = 1, _sync_status = 'synced' WHERE id = ?1",
                        [id],
                    )?;
                    stats.clients_deleted += 1;
                } else {
                    // Upsert from server
                    conn.execute(
                        "INSERT OR REPLACE INTO clients 
                         (id, name, email, phone, created_at, updated_at, version, is_deleted, _sync_status)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 'synced')",
                        rusqlite::params![
                            id,
                            client["name"].as_str().unwrap(),
                            client["email"].as_str().unwrap(),
                            client["phone"].as_str(),
                            client["created_at"].as_str().unwrap(),
                            client["updated_at"].as_str().unwrap(),
                            version,
                        ],
                    )?;
                    stats.clients_pulled += 1;
                }
            } else {
                // Local version is same or newer - keep local
                stats.clients_skipped += 1;
            }
        }
        
        Ok(())
    }
    
    async fn pull_and_merge_properties(&self, conn: &Connection, stats: &mut SyncStats) -> Result<()> {
        let properties = self.pull_properties().await?;
        
        for prop in properties {
            let id = prop["id"].as_str().unwrap();
            let version = prop["version"].as_i64().unwrap_or(1);
            let is_deleted = prop["is_deleted"].as_bool().unwrap_or(false);
            
            let local_version: Option<i64> = conn
                .query_row(
                    "SELECT version FROM properties WHERE id = ?1",
                    [id],
                    |row| row.get(0),
                )
                .ok();
            
            if local_version.is_none() || version > local_version.unwrap() {
                if is_deleted {
                    conn.execute(
                        "UPDATE properties SET is_deleted = 1, _sync_status = 'synced' WHERE id = ?1",
                        [id],
                    )?;
                    stats.properties_deleted += 1;
                } else {
                    conn.execute(
                        "INSERT OR REPLACE INTO properties 
                         (id, client_id, name, address, city, state, zip_code, created_at, updated_at, version, is_deleted, _sync_status)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, 'synced')",
                        rusqlite::params![
                            id,
                            prop["client_id"].as_str().unwrap(),
                            prop["name"].as_str().unwrap(),
                            prop["address"].as_str().unwrap(),
                            prop["city"].as_str(),
                            prop["state"].as_str(),
                            prop["zip_code"].as_str(),
                            prop["created_at"].as_str().unwrap(),
                            prop["updated_at"].as_str().unwrap(),
                            version,
                        ],
                    )?;
                    stats.properties_pulled += 1;
                }
            } else {
                stats.properties_skipped += 1;
            }
        }
        
        Ok(())
    }

    // ========================================
    // HTTP CLIENT METHODS (existing)
    // ========================================
    
    pub async fn pull_clients(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/clients", self.api_url);
        let response = self.client
            .get(&url)
            .header("x-tenant-id", &self.tenant_id)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch clients: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    pub async fn pull_properties(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/properties", self.api_url);
        let response = self.client
            .get(&url)
            .header("x-tenant-id", &self.tenant_id)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch properties: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        Ok(body["data"].as_array().cloned().unwrap_or_default())
    }

    pub async fn push_client(
        &self,
        name: &str,
        email: &str,
        phone: Option<&str>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/clients", self.api_url);
        let payload = serde_json::json!({
            "name": name,
            "email": email,
            "phone": phone,
        });

        let response = self.client
            .post(&url)
            .header("x-tenant-id", &self.tenant_id)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create client: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        Ok(body["data"].clone())
    }

    pub async fn push_property(
        &self,
        client_id: &str,
        name: &str,
        address: &str,
        city: Option<&str>,
        state: Option<&str>,
        zip_code: Option<&str>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/properties", self.api_url);
        let payload = serde_json::json!({
            "client_id": client_id,
            "name": name,
            "address": address,
            "city": city,
            "state": state,
            "zip_code": zip_code,
        });

        let response = self.client
            .post(&url)
            .header("x-tenant-id", &self.tenant_id)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create property: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        Ok(body["data"].clone())
    }

    pub async fn delete_client(&self, id: &str) -> Result<()> {
        let url = format!("{}/clients/{}", self.api_url, id);
        let response = self.client
            .delete(&url)
            .header("x-tenant-id", &self.tenant_id)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to delete client: {}", response.status());
        }
        Ok(())
    }

    pub async fn delete_property(&self, id: &str) -> Result<()> {
        let url = format!("{}/properties/{}", self.api_url, id);
        let response = self.client
            .delete(&url)
            .header("x-tenant-id", &self.tenant_id)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to delete property: {}", response.status());
        }
        Ok(())
    }

    pub async fn check_connection(&self) -> bool {
        let url = "http://localhost:3000/health";
        match self.client
            .get(url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

// ========================================
// SYNC STATISTICS
// ========================================

#[derive(Default)]
struct SyncStats {
    clients_pushed: usize,
    clients_pulled: usize,
    clients_deleted: usize,
    clients_skipped: usize,
    properties_pushed: usize,
    properties_pulled: usize,
    properties_deleted: usize,
    properties_skipped: usize,
    errors: usize,
}

impl SyncStats {
    fn summary(&self) -> String {
        format!(
            "✅ Sync Complete!\n\
             📤 Pushed: {} clients, {} properties\n\
             📥 Pulled: {} clients, {} properties\n\
             🗑️  Deleted: {} clients, {} properties\n\
             ⏭️  Skipped: {} clients, {} properties\n\
             ❌ Errors: {}",
            self.clients_pushed,
            self.properties_pushed,
            self.clients_pulled,
            self.properties_pulled,
            self.clients_deleted,
            self.properties_deleted,
            self.clients_skipped,
            self.properties_skipped,
            self.errors
        )
    }
}