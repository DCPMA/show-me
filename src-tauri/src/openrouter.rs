use serde::{Deserialize, Serialize};

const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

/// A single step of guidance returned by the AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidanceStep {
    /// Step number (1-indexed)
    pub step: u32,
    /// Human-readable instruction
    pub instruction: String,
    /// Optional UI element to highlight
    pub element: Option<UiElement>,
}

/// A UI element the user should interact with
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiElement {
    /// What type of element (button, menu, input, etc.)
    pub kind: String,
    /// Label or description of the element
    pub label: String,
    /// Approximate x coordinate (pixels from left)
    pub x: f64,
    /// Approximate y coordinate (pixels from top)
    pub y: f64,
    /// Approximate width
    pub width: f64,
    /// Approximate height
    pub height: f64,
}

/// Full AI analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    /// Summary of what the user should do
    pub summary: String,
    /// Ordered steps
    pub steps: Vec<GuidanceStep>,
}

/// Error types for OpenRouter API calls
#[derive(Debug)]
pub enum ApiError {
    NoApiKey,
    RequestFailed(String),
    InvalidResponse(String),
    RateLimited,
    InvalidApiKey,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NoApiKey => write!(f, "No OpenRouter API key configured. Go to Settings to add one."),
            ApiError::RequestFailed(msg) => write!(f, "API request failed: {}", msg),
            ApiError::InvalidResponse(msg) => write!(f, "Could not parse AI response: {}", msg),
            ApiError::RateLimited => write!(f, "Rate limited by OpenRouter. Please wait and try again."),
            ApiError::InvalidApiKey => write!(f, "Invalid API key. Check your OpenRouter API key in Settings."),
        }
    }
}

const SYSTEM_PROMPT: &str = r#"You are ShowMe, an AI assistant that analyzes screenshots and provides step-by-step visual guidance.

Given a screenshot and a user's question, respond with a JSON object containing:
1. A brief summary of what the user should do
2. An ordered list of steps, where each step has an instruction and optionally identifies a UI element to interact with

The UI element coordinates should be approximate pixel positions relative to the screenshot's top-left corner.

Respond ONLY with valid JSON in this exact format:
{
  "summary": "Brief description of the solution",
  "steps": [
    {
      "step": 1,
      "instruction": "What to do in this step",
      "element": {
        "kind": "button|menu|input|link|tab|icon|checkbox|other",
        "label": "Text or description of the element",
        "x": 100,
        "y": 200,
        "width": 80,
        "height": 30
      }
    },
    {
      "step": 2,
      "instruction": "A step without a specific UI element to highlight",
      "element": null
    }
  ]
}

Rules:
- Always respond with valid JSON only, no markdown fences or extra text
- Coordinates are in pixels relative to the screenshot
- If a step doesn't involve clicking a specific element, set element to null
- Be precise about element locations
- Keep instructions concise and actionable"#;

/// Send a screenshot and question to OpenRouter for analysis
pub async fn analyze_screenshot(
    api_key: &str,
    model: &str,
    screenshot_b64: &str,
    question: &str,
) -> Result<AnalysisResponse, ApiError> {
    if api_key.is_empty() {
        return Err(ApiError::NoApiKey);
    }

    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": SYSTEM_PROMPT
            },
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": question
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/png;base64,{}", screenshot_b64)
                        }
                    }
                ]
            }
        ],
        "temperature": 0.1,
        "max_tokens": 2048
    });

    let client = reqwest::Client::new();
    let response = client
        .post(OPENROUTER_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("HTTP-Referer", "https://github.com/DCPMA/show-me")
        .header("X-Title", "ShowMe")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(ApiError::InvalidApiKey);
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(ApiError::RateLimited);
    }
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(ApiError::RequestFailed(format!("{}: {}", status, body)));
    }

    let resp_body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| ApiError::InvalidResponse(e.to_string()))?;

    // Extract the assistant's message content
    let content = resp_body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| ApiError::InvalidResponse("No content in response".into()))?;

    // Strip markdown code fences if the model wraps the JSON
    let clean = content
        .trim()
        .strip_prefix("```json")
        .or_else(|| content.trim().strip_prefix("```"))
        .unwrap_or(content.trim())
        .strip_suffix("```")
        .unwrap_or(content.trim())
        .trim();

    let analysis: AnalysisResponse = serde_json::from_str(clean)
        .map_err(|e| ApiError::InvalidResponse(format!("JSON parse error: {}. Raw: {}", e, &clean[..clean.len().min(200)])))?;

    Ok(analysis)
}
