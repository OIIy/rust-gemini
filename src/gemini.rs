// create a http client for gemini using reqwest
// we need the http client to be accessible my multiple threads to allow for full asynchronus
// processing

use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    candidates: Vec<Candidate>,
    usage_metadata: UsageMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Candidate {
    pub content: Content,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
    pub index: usize,
    #[serde(rename = "safetyRatings")]
    pub safety_ratings: Vec<SafetyRating>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SafetyRating {
    pub category: String,
    #[serde(rename = "probability")]
    pub probability: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: usize,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: usize,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Part {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub parts: Vec<Part>,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestBody {
    contents: Vec<Content>,
}

#[derive(Error, Debug)]
pub enum GeminiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct Gemini {
    client: Arc<Client>,
    api_key: String,
    model: String,
}

impl Gemini {
    pub fn new(api_key: Option<&str>, model: Option<&str>) -> Self {
        // test for api_key
        let api_key = api_key
            .map(String::from)
            .expect("An API Key must be provided.");

        // test for model
        let model = model.map(String::from).expect("A model must be specified");

        Gemini {
            client: Arc::new(Client::new()),
            api_key,
            model,
        }
    }

    pub async fn ask(&self, prompt: &str) -> Result<Vec<String>, GeminiError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let body = RequestBody {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
                role: "user".to_string(),
            }],
        };

        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let raw_body = response.text().await?;
        let response_body: Response = serde_json::from_str(&raw_body)?;

        Ok(response_body
            .candidates
            .iter()
            .flat_map(|candidate| candidate.content.parts.iter().map(|part| part.text.clone()))
            .collect())
    }
}
