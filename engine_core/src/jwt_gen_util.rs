use serde::{Deserialize, Serialize};
use reqwest::Client;
use rand::{Rng, thread_rng};

// Function to generate random string
fn generate_random_string(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = thread_rng();
    
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

// Response structure for the session creation
#[derive(Deserialize, Debug)]
struct SessionResponse {
    session_id: String,
    token: String,
    // Add other fields as needed
}

// Request body structure
#[derive(Serialize)]
struct SessionRequest {
    api_secret: String,
}

pub async fn get_jwt_token() -> Result<String, Box<dyn std::error::Error>> {
    // Step 1: Generate random API key
    let api_key = generate_random_string(12);
    
    // Step 2: Generate random API secret
    let api_secret = generate_random_string(24);
    
    // Step 4: Make the RESTful HTTP POST call
    let client = Client::new();
    // Use an appropriate base URL - this example assumes a local server
    let url = format!("http://localhost:8080/api/auth/{}/sessions", api_key);
    
    let request_body = SessionRequest {
        api_secret: api_secret.clone(),
    };
    
    let response = client.post(url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;
    
    // Check if response is successful
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        eprintln!("Failed to create session ({}): {}", status, error_text);
        return Err(format!("HTTP error: {}", status).into());
    }
    
    // Parse the response
    let result: SessionResponse = response.json().await?;
    
    // Step 5: Output the token to the console
    println!("Token: {}", result.token);
    
    Ok(result.token)
}