use crate::types::{Model, SearchMode};

pub const API_BASE_URL: &str = "https://www.perplexity.ai";
pub const API_VERSION: &str = "2.18";

pub const ENDPOINT_AUTH_SESSION: &str = "/api/auth/session";
pub const ENDPOINT_SSE_ASK: &str = "/rest/sse/perplexity_ask";
pub const ENDPOINT_UPLOAD_URL: &str = "/rest/uploads/create_upload_url";

/// Returns the model preference string for the API payload.
///
/// Returns `Some(preference)` if the mode+model combination is valid,
/// or `None` if the model is incompatible with the given mode.
pub fn model_preference(mode: SearchMode, model: Option<Model>) -> Option<&'static str> {
    match (mode, model) {
        // Auto mode - only default model
        (SearchMode::Auto, None) => Some("turbo"),
        (SearchMode::Auto, Some(_)) => None,

        // Pro mode models
        (SearchMode::Pro, None) => Some("pplx_pro"),
        (SearchMode::Pro, Some(Model::Sonar)) => Some("experimental"),
        (SearchMode::Pro, Some(Model::Gpt52)) => Some("gpt52"),
        (SearchMode::Pro, Some(Model::Claude45Sonnet)) => Some("claude45sonnet"),
        (SearchMode::Pro, Some(Model::Grok41)) => Some("grok41nonreasoning"),
        (SearchMode::Pro, Some(_)) => None, // Other models not valid for Pro

        // Reasoning mode models
        (SearchMode::Reasoning, None) => Some("pplx_reasoning"),
        (SearchMode::Reasoning, Some(Model::Gpt52Thinking)) => Some("gpt52_thinking"),
        (SearchMode::Reasoning, Some(Model::Claude45SonnetThinking)) => {
            Some("claude45sonnetthinking")
        }
        (SearchMode::Reasoning, Some(Model::Gemini30Pro)) => Some("gemini30pro"),
        (SearchMode::Reasoning, Some(Model::KimiK2Thinking)) => Some("kimik2thinking"),
        (SearchMode::Reasoning, Some(Model::Grok41Reasoning)) => Some("grok41reasoning"),
        (SearchMode::Reasoning, Some(_)) => None, // Other models not valid for Reasoning

        // Deep Research mode - only default model
        (SearchMode::DeepResearch, None) => Some("pplx_alpha"),
        (SearchMode::DeepResearch, Some(_)) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_mode_defaults() {
        assert_eq!(model_preference(SearchMode::Auto, None), Some("turbo"));
    }

    #[test]
    fn test_auto_mode_rejects_models() {
        assert_eq!(model_preference(SearchMode::Auto, Some(Model::Gpt52)), None);
        assert_eq!(model_preference(SearchMode::Auto, Some(Model::Sonar)), None);
    }

    #[test]
    fn test_pro_mode_defaults() {
        assert_eq!(model_preference(SearchMode::Pro, None), Some("pplx_pro"));
    }

    #[test]
    fn test_pro_mode_models() {
        assert_eq!(
            model_preference(SearchMode::Pro, Some(Model::Sonar)),
            Some("experimental")
        );
        assert_eq!(model_preference(SearchMode::Pro, Some(Model::Gpt52)), Some("gpt52"));
        assert_eq!(
            model_preference(SearchMode::Pro, Some(Model::Claude45Sonnet)),
            Some("claude45sonnet")
        );
        assert_eq!(
            model_preference(SearchMode::Pro, Some(Model::Grok41)),
            Some("grok41nonreasoning")
        );
    }

    #[test]
    fn test_pro_mode_rejects_reasoning_models() {
        assert_eq!(model_preference(SearchMode::Pro, Some(Model::Gpt52Thinking)), None);
        assert_eq!(
            model_preference(SearchMode::Pro, Some(Model::Claude45SonnetThinking)),
            None
        );
    }

    #[test]
    fn test_reasoning_mode_defaults() {
        assert_eq!(model_preference(SearchMode::Reasoning, None), Some("pplx_reasoning"));
    }

    #[test]
    fn test_reasoning_mode_models() {
        assert_eq!(
            model_preference(SearchMode::Reasoning, Some(Model::Gpt52Thinking)),
            Some("gpt52_thinking")
        );
        assert_eq!(
            model_preference(SearchMode::Reasoning, Some(Model::Claude45SonnetThinking)),
            Some("claude45sonnetthinking")
        );
        assert_eq!(
            model_preference(SearchMode::Reasoning, Some(Model::Gemini30Pro)),
            Some("gemini30pro")
        );
        assert_eq!(
            model_preference(SearchMode::Reasoning, Some(Model::KimiK2Thinking)),
            Some("kimik2thinking")
        );
        assert_eq!(
            model_preference(SearchMode::Reasoning, Some(Model::Grok41Reasoning)),
            Some("grok41reasoning")
        );
    }

    #[test]
    fn test_reasoning_mode_rejects_pro_models() {
        assert_eq!(model_preference(SearchMode::Reasoning, Some(Model::Gpt52)), None);
        assert_eq!(model_preference(SearchMode::Reasoning, Some(Model::Sonar)), None);
    }

    #[test]
    fn test_deep_research_mode_defaults() {
        assert_eq!(model_preference(SearchMode::DeepResearch, None), Some("pplx_alpha"));
    }

    #[test]
    fn test_deep_research_mode_rejects_models() {
        assert_eq!(model_preference(SearchMode::DeepResearch, Some(Model::Gpt52)), None);
        assert_eq!(
            model_preference(SearchMode::DeepResearch, Some(Model::Gpt52Thinking)),
            None
        );
    }
}
