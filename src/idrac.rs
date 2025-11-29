use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use log::{info, error};
use base64::Engine;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdracError {
    pub message: String,
}

#[derive(Clone)]
pub struct IdracClient {
    base_url: String,
    username: String,
    password: String,
    client: Client,
}

impl IdracClient {
    pub fn from_env() -> Result<Self, String> {
        let base_url = std::env::var("IDRAC_HOST")
            .map_err(|_| "IDRAC_HOST environment variable not set".to_string())?;
        let username = std::env::var("IDRAC_USERNAME")
            .map_err(|_| "IDRAC_USERNAME environment variable not set".to_string())?;
        let password = std::env::var("IDRAC_PASSWORD")
            .map_err(|_| "IDRAC_PASSWORD environment variable not set".to_string())?;

        // Build client that accepts self-signed certificates (common for iDRAC)
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        info!("iDRAC client initialized for host: {}", base_url);
        
        Ok(IdracClient {
            base_url,
            username,
            password,
            client,
        })
    }

    fn get_auth_header(&self) -> String {
        let credentials = format!("{}:{}", self.username, self.password);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        format!("Basic {}", encoded)
    }

    pub async fn get_power_state(&self) -> Result<String, String> {
        let url = format!(
            "{}/redfish/v1/Systems/System.Embedded.1",
            self.base_url
        );

        let response = self.client
            .get(&url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to connect to iDRAC: {}", e))?;

        if response.status() == StatusCode::OK {
            let data: serde_json::Value = response.json().await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            
            let power_state = data["PowerState"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string();
            
            info!("Current power state: {}", power_state);
            Ok(power_state)
        } else {
            let error_msg = format!("Failed to get power state: HTTP {}", response.status());
            error!("{}", error_msg);
            Err(error_msg)
        }
    }

    pub async fn power_on(&self) -> Result<String, String> {
        self.set_power_state("On").await
    }

    pub async fn power_off(&self) -> Result<String, String> {
        self.set_power_state("ForceOff").await
    }

    pub async fn graceful_shutdown(&self) -> Result<String, String> {
        self.set_power_state("GracefulShutdown").await
    }

    async fn set_power_state(&self, reset_type: &str) -> Result<String, String> {
        let url = format!(
            "{}/redfish/v1/Systems/System.Embedded.1/Actions/ComputerSystem.Reset",
            self.base_url
        );

        let payload = serde_json::json!({
            "ResetType": reset_type
        });

        info!("Sending power command: {}", reset_type);

        let response = self.client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to iDRAC: {}", e))?;

        if response.status() == StatusCode::NO_CONTENT || response.status() == StatusCode::OK {
            let success_msg = format!("Successfully executed: {}", reset_type);
            info!("{}", success_msg);
            Ok(success_msg)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            let error_msg = format!("Failed to set power state: HTTP {} - {}", status, error_text);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}
