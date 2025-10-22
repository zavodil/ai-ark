use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Read, Write};
use std::time::Duration;
use wasi_http_client::Client;

#[derive(Deserialize, Debug)]
struct Input {
    prompt: String,
    #[serde(default)]
    history: Vec<Message>,
    #[serde(default = "default_endpoint")]
    openai_endpoint: String,
    #[serde(default = "default_model")]
    model_name: String,
    #[serde(default = "default_max_tokens")]
    max_tokens: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

fn default_endpoint() -> String {
    "https://api.openai.com/v1/chat/completions".to_string()
}

fn default_model() -> String {
    "gpt-3.5-turbo".to_string()
}

fn default_max_tokens() -> u32 {
    150
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read input from stdin
    let mut input_string = String::new();
    io::stdin().read_to_string(&mut input_string)?;

    // Parse input JSON
    let input: Input = serde_json::from_str(&input_string)?;

    // Get API key from environment variable
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: OPENAI_API_KEY not found in environment variables");
            std::process::exit(1);
        }
    };

    // Build messages array
    let mut messages = input.history;
    messages.push(Message {
        role: "user".to_string(),
        content: input.prompt,
    });

    // Create OpenAI request
    let request_body = OpenAIRequest {
        model: input.model_name,
        messages,
        max_tokens: input.max_tokens,
        temperature: 0.7,
    };

    // Serialize request to JSON
    let request_json = serde_json::to_string(&request_body)?;

    // Make HTTPS request using wasi-http-client
    let response = Client::new()
        .post(&input.openai_endpoint)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .connect_timeout(Duration::from_secs(30))
        .body(request_json.as_bytes())
        .send()?;

    // Check response status
    let status = response.status();
    if status < 200 || status >= 300 {
        match response.body() {
            Ok(body_bytes) => {
                let error_text = String::from_utf8_lossy(&body_bytes);
                eprintln!(
                    "Error: OpenAI API returned status {}. Details: {}",
                    status, error_text
                );
            }
            Err(e) => {
                eprintln!("Error: OpenAI API returned status {}. Failed to read body: {:?}", status, e);
            }
        }
        std::process::exit(1);
    }

    // Parse response
    let response_body = response.body()?;
    let response_data: OpenAIResponse = serde_json::from_slice(&response_body)?;

    // Extract and print answer
    if let Some(choice) = response_data.choices.first() {
        print!("{}", choice.message.content);
        io::stdout().flush()?;
    } else {
        eprintln!("Error: No response from OpenAI API");
        std::process::exit(1);
    }

    Ok(())
}