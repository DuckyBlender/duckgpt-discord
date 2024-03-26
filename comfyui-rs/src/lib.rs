use futures_util::StreamExt;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

pub struct Client {
    server_address: String,
    client_id: String,
}

impl Client {
    pub fn new(server_address: &str) -> Self {
        let client_id = Uuid::new_v4().to_string();
        Client {
            server_address: server_address.to_string(),
            client_id,
        }
    }

    pub async fn queue_prompt(
        &self,
        prompt: serde_json::Value,
    ) -> Result<serde_json::Value, reqwest::Error> {
        info!("Queueing prompt");
        let client = reqwest::Client::new();
        let p = json!({
            "prompt": prompt,
            "client_id": self.client_id
        });

        let res = client
            .post(format!("http://{}/prompt", self.server_address))
            .json(&p)
            .send()
            .await?;

        // Convert res to json
        let response: serde_json::Value = res.json().await?;
        Ok(response)
    }

    pub async fn get_image(&self, filename: &str) -> Result<Vec<u8>, reqwest::Error> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://{}/view", self.server_address))
            .query(&[("filename", filename)])
            .send()
            .await?;

        // Print the dimensions of the image
        let bytes = res.bytes().await?;
        info!("Got image: {:?} bytes", bytes.len());
        Ok(bytes.to_vec())
    }

    pub async fn get_history(&self, prompt_id: &str) -> Result<serde_json::Value, reqwest::Error> {
        info!("Getting history for prompt_id: {prompt_id}");
        let client = reqwest::Client::new();
        let res = client
            .get(format!(
                "http://{}/history/{}",
                self.server_address, prompt_id
            ))
            .send()
            .await?;

        // Print .text()
        let res = res.json().await?;
        info!("Got history for prompt_id: {prompt_id}");

        Ok(res)
    }

    // this currently gets one image lmao
    pub async fn get_images(
        &self,
        json_prompt: serde_json::Value,
    ) -> Result<HashMap<String, Vec<u8>>, Box<dyn std::error::Error>> {
        let prompt_id = self.queue_prompt(json_prompt).await?;

        info!("prompt_id: {prompt_id:?}");

        let prompt_id = prompt_id["prompt_id"].as_str().unwrap().to_string();
        let mut images: HashMap<String, Vec<u8>> = HashMap::new(); // image name and bytes

        // WebSocket connection setup
        let (ws_stream, _) = tokio_tungstenite::connect_async(format!(
            "ws://{}/ws?clientId={}",
            self.server_address, self.client_id
        ))
        .await?;
        let (mut _write, mut read) = ws_stream.split();

        // WebSocket message loop
        while let Some(message) = read.next().await {
            let msg = message?;
            if msg.is_text() {
                let message: HashMap<String, serde_json::Value> =
                    serde_json::from_str(msg.to_text().unwrap())?;
                if message["type"] == "executing"
                    && message["data"]["node"].is_null()
                    && message["data"]["prompt_id"].as_str().unwrap() == prompt_id
                {
                    break; // Execution is done
                }
            }
        }

        info!("Fetching history and images");

        // Fetch history and images
        let history = self.get_history(&prompt_id).await?;
        for (_, value) in history[prompt_id.clone()]["outputs"].as_object().unwrap() {
            if value["images"].is_array() {
                for image in value["images"].as_array().unwrap() {
                    let filename = image["filename"].as_str().unwrap();
                    let image = self.get_image(filename).await?;
                    // Check if image is empty
                    if image.len() == 0 {
                        continue;
                    }
                    images.insert(filename.to_string(), image);
                }
            }
        }
        Ok(images)
    }

    pub async fn get_system_stats(&self) -> Result<SystemStats, reqwest::Error> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://{}/system_stats", self.server_address))
            .send()
            .await?;
        let response: SystemStats = res.json().await?;
        Ok(response)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemStats {
    pub system: System,
    pub devices: Vec<Device>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct System {
    pub os: String,
    pub python_version: String,
    pub embedded_python: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
    pub name: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub index: u32,
    pub vram_total: u64,
    pub vram_free: u64,
    pub torch_vram_total: u64,
    pub torch_vram_free: u64,
}
