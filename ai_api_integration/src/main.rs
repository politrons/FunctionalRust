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
        "https://generativelanguage.googleapis.com/v1/models/gemini-1.5-pro:generateContent?key={}",
        api_key
    );

    let client = Client::new();

    // Adapted request body based on the curl command
    let request_body = ModelRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: "what is your latest data date.".to_string(),
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

        // Extracting the text from the JSON response
        if let Some(text) = response_json["candidates"]
            .get(0)
            .and_then(|candidate| candidate["content"]["parts"].get(0))
            .and_then(|part| part["text"].as_str())
        {
            println!("\nGenerated text: {}", text);
        } else {
            println!("Text not found in the response");
        }
    } else {
        println!("Request failed with status: {}", response.status());
    }


    Ok(())
}
