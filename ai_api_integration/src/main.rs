use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use warp::Filter;
use serde::{Deserialize, Serialize};
use reqwest::Client;

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
struct Question {
    question: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = "AIzaSyDUZRX8uEI1VSARyHMA3s6HjEE-5OK4-vw";
    let model_url = format!(
        "https://generativelanguage.googleapis.com/v1/models/gemini-1.5-pro:generateContent?key={}",
        api_key
    );

    let client = Client::new();

    // Serve the HTML file
    let index = warp::path::end()
        .and(warp::fs::file("index.html"));

    // Handle POST requests to /ask
    let ask = warp::path("ask")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(move |question: Question| {
            let client = client.clone();
            let model_url = model_url.clone();
            println!("Question {:?}", question.question);
            async move {
                let request_body = ModelRequest {
                    contents: vec![Content {
                        parts: vec![Part {
                            text: question.question,
                        }],
                    }],
                };

                let response = client
                    .post(&model_url)
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await;

                match response {
                    Ok(res) if res.status().is_success() => {
                        let response_json: serde_json::Value = res.json().await.unwrap();

                        let text = response_json["candidates"]
                            .get(0)
                            .and_then(|candidate| candidate["content"]["parts"].get(0))
                            .and_then(|part| part["text"].as_str())
                            .unwrap_or("No response text found")
                            .to_string();

                        Ok(warp::reply::json(&serde_json::json!({ "response": text })))
                            as Result<_, Infallible>
                    }
                    _ => Ok(warp::reply::json(&serde_json::json!({ "response": "Error occurred" }))),
                }
            }
        });

    // Combine routes
    let routes = index.or(ask);

    // Start the server
    let addr: SocketAddr = "127.0.0.1:3030".parse()?;
    warp::serve(routes).run(addr).await;

    Ok(())
}
