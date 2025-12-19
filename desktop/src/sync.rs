use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

const API_BASE_URL: &str = "http://localhost:3000/api";
const TENANT_ID: &str = "00000000-0000-0000-0000-000000000001"; // Demo tenant

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncConfig {
    pub api_url: String,
    pub tenant_id: String,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            api_url: API_BASE_URL.to_string(),
            tenant_id: TENANT_ID.to_string(),
        }
    }
}

pub struct SyncService {
    config: SyncConfig,
    client: reqwest::Client,
}

impl SyncService {
    pub fn new() -> Self {
        Self {
            config: SyncConfig::default(),
            client: reqwest::Client::new(),
        }
    }

    // Pull clients from server
    pub async fn pull_clients(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/clients", self.config.api_url);
        
        let response = self.client
            .get(&url)
            .header("x-tenant-id", &self.config.tenant_id)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch clients: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        
        // Extract data from ApiResponse wrapper
        if let Some(data) = body.get("data") {
            if let Some(clients) = data.as_array() {
                return Ok(clients.clone());
            }
        }
        
        Ok(vec![])
    }

    // Pull properties from server
    pub async fn pull_properties(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/properties", self.config.api_url);
        
        let response = self.client
            .get(&url)
            .header("x-tenant-id", &self.config.tenant_id)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch properties: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        
        if let Some(data) = body.get("data") {
            if let Some(properties) = data.as_array() {
                return Ok(properties.clone());
            }
        }
        
        Ok(vec![])
    }

    // Push client to server
    pub async fn push_client(
        &self,
        name: &str,
        email: &str,
        phone: Option<&str>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/clients", self.config.api_url);
        
        let payload = serde_json::json!({
            "name": name,
            "email": email,
            "phone": phone,
        });

        let response = self.client
            .post(&url)
            .header("x-tenant-id", &self.config.tenant_id)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create client: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        
        if let Some(data) = body.get("data") {
            return Ok(data.clone());
        }
        
        anyhow::bail!("No data in response")
    }

    // Push property to server
    pub async fn push_property(
        &self,
        client_id: &str,
        name: &str,
        address: &str,
        city: Option<&str>,
        state: Option<&str>,
        zip_code: Option<&str>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/properties", self.config.api_url);
        
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
            .header("x-tenant-id", &self.config.tenant_id)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create property: {}", response.status());
        }

        let body: serde_json::Value = response.json().await?;
        
        if let Some(data) = body.get("data") {
            return Ok(data.clone());
        }
        
        anyhow::bail!("No data in response")
    }

    // Delete client on server
    pub async fn delete_client(&self, id: &str) -> Result<()> {
        let url = format!("{}/clients/{}", self.config.api_url, id);
        
        let response = self.client
            .delete(&url)
            .header("x-tenant-id", &self.config.tenant_id)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to delete client: {}", response.status());
        }

        Ok(())
    }

    // Check if server is reachable
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
