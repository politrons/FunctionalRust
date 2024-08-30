use std::error::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelRequest {
    contents: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelResponse {
    generated_text: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Replace with your Google API key
    let api_key = "AIzaSyDUZRX8uEI1VSARyHMA3s6HjEE-5OK4-vw";
    let model_url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
        api_key
    );

    let client = Client::new();

    // Adapted request body based on the curl command
    let request_body = ModelRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: "Who won last champion league.".to_string(),
            }],
        }],
    };

    let response = client
        .post(&model_url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        let response_json: serde_json::Value = response.json().await?;

        // Print the whole JSON response to inspect it
        println!("\nFull JSON response: {:#?}", response_json);

        // Example: Access specific fields (adjust based on the actual structure)
        // let output = response_json["some_field"].as_str().unwrap_or("No response text found");
    } else {
        println!("Request failed with status: {}", response.status());
    }

    Ok(())
}
