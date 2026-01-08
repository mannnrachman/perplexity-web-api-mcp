use crate::error::{Error, Result};
use crate::types::SearchEvent;
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Keys that are extracted from the raw JSON and stored in dedicated fields.
const EXTRACTED_KEYS: &[&str] = &["answer", "chunks", "backend_uuid", "attachments"];

/// Parses an SSE event JSON string into a SearchEvent.
pub(crate) fn parse_sse_event(json_str: &str) -> Result<SearchEvent> {
    let mut content: Map<String, Value> =
        serde_json::from_str(json_str).map_err(Error::Json)?;

    // Try to parse the "text" field if it contains nested JSON
    parse_nested_text_field(&mut content);

    // Extract answer and chunks from the FINAL step or fall back to top-level
    let (answer, chunks) = extract_answer_and_chunks(&content);

    // Extract other known fields
    let backend_uuid = extract_string(&content, "backend_uuid");
    let attachments = extract_string_array(&content, "attachments");

    // Build raw map excluding extracted keys
    let raw = build_raw_map(content);

    Ok(SearchEvent { answer, chunks, backend_uuid, attachments, raw })
}

/// If the "text" field is a JSON string, parse it and replace the field with the parsed value.
fn parse_nested_text_field(content: &mut Map<String, Value>) {
    let Some(text_value) = content.get("text") else {
        return;
    };

    let Some(text_str) = text_value.as_str() else {
        return;
    };

    if let Ok(parsed) = serde_json::from_str::<Value>(text_str) {
        content.insert("text".to_string(), parsed);
    }
}

/// Extracts answer and chunks from the event content.
///
/// First tries to find them in a FINAL step within the "text" field,
/// then falls back to top-level "answer" and "chunks" fields.
fn extract_answer_and_chunks(content: &Map<String, Value>) -> (Option<String>, Vec<Value>) {
    // Try to extract from FINAL step in text field
    if let Some((answer, chunks)) = extract_from_final_step(content) {
        return (answer, chunks);
    }

    // Fall back to top-level fields
    let answer = extract_string(content, "answer");
    let chunks = content.get("chunks").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    (answer, chunks)
}

/// Extracts answer and chunks from a FINAL step in the text field.
fn extract_from_final_step(
    content: &Map<String, Value>,
) -> Option<(Option<String>, Vec<Value>)> {
    let text = content.get("text")?;
    let steps = text.as_array()?;

    let final_step = steps
        .iter()
        .find(|step| step.get("step_type").and_then(|v| v.as_str()) == Some("FINAL"))?;

    let step_content = final_step.get("content")?;
    let answer_str = step_content.get("answer")?.as_str()?;

    let answer_data: Value = serde_json::from_str(answer_str).ok()?;

    let answer = answer_data.get("answer").and_then(|v| v.as_str()).map(|s| s.to_string());

    let chunks =
        answer_data.get("chunks").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    Some((answer, chunks))
}

/// Extracts a string value from the content map.
fn extract_string(content: &Map<String, Value>, key: &str) -> Option<String> {
    content.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Extracts an array of strings from the content map.
fn extract_string_array(content: &Map<String, Value>, key: &str) -> Vec<String> {
    content
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

/// Builds the raw map by excluding extracted keys.
fn build_raw_map(content: Map<String, Value>) -> HashMap<String, Value> {
    content.into_iter().filter(|(k, _)| !EXTRACTED_KEYS.contains(&k.as_str())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_event() {
        let json = r#"{"answer": "Hello world", "chunks": [{"title": "Source 1"}]}"#;
        let event = parse_sse_event(json).unwrap();

        assert_eq!(event.answer, Some("Hello world".to_string()));
        assert_eq!(event.chunks.len(), 1);
        assert!(event.backend_uuid.is_none());
        assert!(event.attachments.is_empty());
    }

    #[test]
    fn test_parse_event_with_backend_uuid() {
        let json = r#"{"answer": "Test", "backend_uuid": "abc-123", "chunks": []}"#;
        let event = parse_sse_event(json).unwrap();

        assert_eq!(event.answer, Some("Test".to_string()));
        assert_eq!(event.backend_uuid, Some("abc-123".to_string()));
    }

    #[test]
    fn test_parse_event_with_attachments() {
        let json = r#"{"answer": "Test", "attachments": ["url1", "url2"], "chunks": []}"#;
        let event = parse_sse_event(json).unwrap();

        assert_eq!(event.attachments, vec!["url1", "url2"]);
    }

    #[test]
    fn test_parse_event_with_nested_text_json() {
        // Simulates the "text" field containing JSON string with steps
        let inner_answer = r#"{"answer": "Nested answer", "chunks": [{"id": 1}]}"#;
        let text_content = serde_json::json!([
            {
                "step_type": "SEARCH",
                "content": {}
            },
            {
                "step_type": "FINAL",
                "content": {
                    "answer": inner_answer
                }
            }
        ]);
        let text_str = serde_json::to_string(&text_content).unwrap();

        let json = serde_json::json!({
            "text": text_str,
            "some_field": "value"
        });

        let event = parse_sse_event(&json.to_string()).unwrap();

        assert_eq!(event.answer, Some("Nested answer".to_string()));
        assert_eq!(event.chunks.len(), 1);
        // The "text" field should be parsed and stored in raw
        assert!(event.raw.contains_key("text"));
        assert!(event.raw.contains_key("some_field"));
    }

    #[test]
    fn test_parse_event_fallback_to_top_level() {
        // When text doesn't contain FINAL step, fall back to top-level
        let text_content = serde_json::json!([
            {
                "step_type": "SEARCH",
                "content": {}
            }
        ]);
        let text_str = serde_json::to_string(&text_content).unwrap();

        let json = serde_json::json!({
            "text": text_str,
            "answer": "Top level answer",
            "chunks": [{"source": "web"}]
        });

        let event = parse_sse_event(&json.to_string()).unwrap();

        assert_eq!(event.answer, Some("Top level answer".to_string()));
        assert_eq!(event.chunks.len(), 1);
    }

    #[test]
    fn test_parse_event_raw_excludes_extracted_keys() {
        let json = r#"{
            "answer": "Test",
            "chunks": [],
            "backend_uuid": "uuid",
            "attachments": [],
            "extra_field": "should be in raw",
            "another": 123
        }"#;
        let event = parse_sse_event(json).unwrap();

        // Extracted keys should not be in raw
        assert!(!event.raw.contains_key("answer"));
        assert!(!event.raw.contains_key("chunks"));
        assert!(!event.raw.contains_key("backend_uuid"));
        assert!(!event.raw.contains_key("attachments"));

        // Other fields should be in raw
        assert!(event.raw.contains_key("extra_field"));
        assert!(event.raw.contains_key("another"));
    }

    #[test]
    fn test_parse_event_empty_fields() {
        let json = r#"{}"#;
        let event = parse_sse_event(json).unwrap();

        assert!(event.answer.is_none());
        assert!(event.chunks.is_empty());
        assert!(event.backend_uuid.is_none());
        assert!(event.attachments.is_empty());
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_sse_event("not json");
        assert!(result.is_err());
    }
}
