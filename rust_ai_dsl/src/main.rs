use serde::{Deserialize, Serialize};

fn main() {
    // Example usage of the RustAIClient to interact with the AI model.
    // It initializes the client with an API token, sets the model and input text,
    // and then sends a request to the AI model, printing the response.
    let response = RustAIClient::with_token("AIzaSyDUZRX8uEI1VSARyHMA3s6HjEE-5OK4-vw".to_string())
        .with_ai_model("Gemini".to_string())
        .with_text("hello".to_string())
        .ask();
    println!("{}", response);
}

// Trait defining the interface for interacting with the AI model.
trait RustAI {
    /// Initializes the RustAIClient with the provided API token.
    ///
    /// # Arguments
    ///
    /// * `token` - A string containing the API token.
    ///
    /// # Returns
    ///
    /// A `RustAIClient` instance initialized with the given API token.
    fn with_token(token: String) -> RustAIClient;

    /// Sets the AI model to be used by the client.
    ///
    /// # Arguments
    ///
    /// * `model` - A string specifying the AI model to use (e.g., "Gemini").
    ///
    /// # Returns
    ///
    /// A new `RustAIClient` instance with the model set.
    fn with_ai_model(&self, model: String) -> RustAIClient;

    /// Sets the input text for the AI model.
    ///
    /// # Arguments
    ///
    /// * `text` - A string containing the input text to be sent to the AI model.
    ///
    /// # Returns
    ///
    /// A new `RustAIClient` instance with the input text set.
    fn with_text(&self, text: String) -> RustAIClient;

    /// Sends a request to the AI model and returns the response as a string.
    ///
    /// # Returns
    ///
    /// A string containing the AI model's response.
    fn ask(self) -> String;
}

// Struct representing the AI client with the necessary parameters.
#[derive(Debug, Serialize, Deserialize)]
struct RustAIClient {
    token: String,
    model: String,
    text: String,
}

//Implementation of all functions defined in the trait 
impl RustAI for RustAIClient {
    fn with_token(_token: String) -> RustAIClient {
        RustAIClient { token: _token, text: "".to_string(), model: "".to_string() }
    }

    fn with_ai_model(&self, _model: String) -> RustAIClient {
        RustAIClient { token: self.token.clone(), model: _model, text: self.text.clone() }
    }

    fn with_text(&self, _text: String) -> RustAIClient {
        RustAIClient { token: self.token.clone(), model: self.model.clone(), text: _text }
    }

    fn ask(self) -> String {
        // Constructs the API endpoint URL based on the selected AI model.
        let model_url = match self.model.as_str() {
            "Gemini" => format!(
                "https://generativelanguage.googleapis.com/v1/models/gemini-1.5-pro:generateContent?key={}",
                self.token
            ),
            _ => return "No Model available".to_string(),
        };

        // Creates the request body with the input text.
        let request_body = ModelRequest {
            contents: vec![Content {
                parts: vec![Part { text: self.text }],
            }],
        };

        // Sends a POST request to the API endpoint and handles the response.
        let post_response = ureq::post(&model_url)
            .send_json(request_body);

        // Parses the JSON response and extracts the AI model's generated text.
        let response_json: serde_json::Value = post_response.unwrap().into_json().unwrap();

        let res = response_json["candidates"].get(0)
            .and_then(|candidate| candidate["content"]["parts"].get(0))
            .and_then(|part| part["text"].as_str()).unwrap();

        res.to_string()
    }
}

// Structs representing the structure of the request body for the API.
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
