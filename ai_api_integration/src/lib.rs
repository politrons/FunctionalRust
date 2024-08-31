use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, Response, RequestInit, Headers};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wasm_bindgen::__rt::IntoJsResult;

#[wasm_bindgen]
pub async fn ask_question(api_key: String, question: String) -> Result<JsValue, JsValue> {
    println!("Request received");

    let model_url = format!(
        "https://generativelanguage.googleapis.com/v1/models/gemini-1.5-flash:generateContent?key={}",
        api_key
    );

    let request_body = ModelRequest {
        contents: vec![Content {
            parts: vec![Part { text: question }],
        }],
    };

    let request = create_request(&model_url, &request_body);
    let resp_value = make_http_request(&request).await;
    let response_json = transform_response_in_json(resp_value).await;
    let text = get_text_from_json(response_json);

    Ok(JsValue::from_str(&text))
}


// Function to create an HTTP request
fn create_request(model_url: &String, request_body: &ModelRequest) -> Request {
    // Serializes the request body to a JSON string
    let body = json!(request_body).to_string();

    let request_init = RequestInit::new();
    request_init.set_method("POST");
    request_init.set_body(&JsValue::from_str(&body));

    let headers = Headers::new().unwrap();
    headers.set("Content-Type", "application/json").unwrap();

    request_init.set_headers(&headers);

    // Creates and returns a new Request object with the specified URL and options
    let request = Request::new_with_str_and_init(&model_url, &request_init).unwrap();
    request
}

// Function to send the HTTP request and receive the response
async fn make_http_request(request: &Request) -> JsValue {
    // Gets the global Window object, which represents the browser window
    let window = web_sys::window().unwrap();

    // Sends the request using the Fetch API to the Google Gemini AI API and waits for the response
    let resp_js_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();
    resp_js_value
}

// Function to convert the response to JSON
async fn transform_response_in_json(resp_value: JsValue) -> Value {
    let resp: Response = resp_value.dyn_into().unwrap();

    // Converts the Response object into a JavaScript Promise that resolves to a JSON object
    let json = JsFuture::from(resp.json().unwrap()).await.unwrap();

    // Converts the JSON object from JsValue to a serde_json::Value for easier manipulation in Rust
    let response_json: Value = json.into_serde().unwrap();
    response_json
}

// Function to extract the relevant text from the JSON response provided by the Google Gemini AI API
fn get_text_from_json(response_json: Value) -> String {
    // Navigates through the JSON structure to extract the text response
    let text = response_json["candidates"]
        .get(0)
        .and_then(|candidate| candidate["content"]["parts"].get(0))
        .and_then(|part| part["text"].as_str())
        .unwrap_or("No response text found")
        .to_string();
    text
}

// Data structures used to represent the request body sent to the Google Gemini AI API
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
