use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, Response,RequestInit,Headers};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wasm_bindgen::__rt::IntoJsResult;

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

#[wasm_bindgen]
pub async fn ask_question(question: String) -> Result<JsValue, JsValue> {
    println!("Request received");
    let api_key = "AIzaSyDUZRX8uEI1VSARyHMA3s6HjEE-5OK4-vw";
    let model_url = format!(
        "https://generativelanguage.googleapis.com/v1/models/gemini-1.5-flash:generateContent?key={}",
        api_key
    );

    let request_body = ModelRequest {
        contents: vec![Content {
            parts: vec![Part { text: question }],
        }],
    };
    let body = json!(request_body).to_string();

    // Construct the Request with the necessary headers
    let request_init = RequestInit::new();
    request_init.set_method("POST");
    request_init.set_body(&JsValue::from_str(&body));

    let headers = Headers::new().unwrap();
    headers.set("Content-Type", "application/json").unwrap();
    // headers.set("Authorization", &format!("Bearer {}", api_key)).unwrap();
    request_init.set_headers(&headers);

    let request = Request::new_with_str_and_init(&model_url, &request_init).unwrap();

    // Fetch the request
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();

    // Convert the response into a Response object
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert the Response into JSON
    let json = JsFuture::from(resp.json().unwrap()).await.unwrap();

    // Directly parse the JSON response into a serde_json::Value
    let response_json: Value = json.into_serde().unwrap();

    // Extract the response text
    let text = response_json["candidates"]
        .get(0)
        .and_then(|candidate| candidate["content"]["parts"].get(0))
        .and_then(|part| part["text"].as_str())
        .unwrap_or("No response text found")
        .to_string();

    Ok(JsValue::from_str(&text))

}