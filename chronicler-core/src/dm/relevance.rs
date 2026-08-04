//! Relevance checking for surfacing appropriate context.
//!
//! Uses the configured provider's fast model to determine which stored consequences
//! and facts are relevant to the current player input, enabling semantic
//! matching instead of just keyword matching.

use super::story_memory::{ConsequenceId, EntityId, FactId, StoryMemory};
use chronicler_llm::{Client, Message, Request};
use serde::Deserialize;
use thiserror::Error;

/// Maximum tokens for relevance check response.
const RELEVANCE_MAX_TOKENS: usize = 500;

/// Errors from relevance checking.
#[derive(Debug, Error)]
pub enum RelevanceError {
    #[error("API error: {0}")]
    ApiError(#[from] chronicler_llm::Error),

    #[error("Failed to parse relevance response: {0}")]
    ParseError(String),
}

/// Result of a relevance check.
#[derive(Debug, Clone, Default)]
pub struct RelevanceResult {
    /// Consequence IDs that should trigger based on current context.
    pub triggered_consequences: Vec<ConsequenceId>,

    /// Fact IDs that are relevant to surface in context.
    pub relevant_facts: Vec<FactId>,

    /// Entity IDs that are relevant but weren't explicitly mentioned.
    pub relevant_entities: Vec<EntityId>,

    /// Raw explanation from the model (for debugging).
    pub explanation: Option<String>,
}

impl RelevanceResult {
    /// Check if any consequences were triggered.
    pub fn has_triggered_consequences(&self) -> bool {
        !self.triggered_consequences.is_empty()
    }

    /// Check if any relevant context was found.
    pub fn has_relevant_context(&self) -> bool {
        !self.relevant_facts.is_empty() || !self.relevant_entities.is_empty()
    }

    /// Check if this result is empty (nothing relevant found).
    pub fn is_empty(&self) -> bool {
        self.triggered_consequences.is_empty()
            && self.relevant_facts.is_empty()
            && self.relevant_entities.is_empty()
    }
}

/// Response format expected from the fast model.
#[derive(Debug, Deserialize)]
struct RelevanceResponse {
    #[serde(default)]
    triggered_consequences: Vec<String>,
    #[serde(default)]
    relevant_entities: Vec<String>,
    #[serde(default)]
    explanation: Option<String>,
}

/// Checks relevance of stored consequences and facts against player input.
pub struct RelevanceChecker {
    client: Client,
    model: String,
}

impl RelevanceChecker {
    /// Create a new relevance checker with the given API client.
    pub fn new(client: Client) -> Self {
        let model = client.fast_model().to_string();
        Self { client, model }
    }

    /// Create from the configured provider environment.
    pub fn from_env() -> Result<Self, chronicler_llm::Error> {
        let client = Client::from_env()?;
        Ok(Self::new(client))
    }

    /// Set a custom model for relevance checking.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Check relevance of stored context against player input.
    ///
    /// This uses the configured fast model to determine:
    /// 1. Which pending consequences should trigger
    /// 2. Which entities are semantically relevant (even if not mentioned by name)
    pub async fn check_relevance(
        &self,
        player_input: &str,
        current_location: &str,
        story_memory: &StoryMemory,
    ) -> Result<RelevanceResult, RelevanceError> {
        // Get pending consequences
        let consequences = story_memory.pending_consequences_by_importance();

        // If no consequences to check, return early
        if consequences.is_empty() {
            return Ok(RelevanceResult::default());
        }

        // Build the consequences list for the prompt
        let consequences_text = story_memory.build_consequences_for_relevance();

        // Build the prompt
        let prompt = format!(
            r#"You are checking if any pending consequences should trigger based on a player's action in a D&D game.

## Player Action
"{player_input}"

## Current Location
{current_location}

## Pending Consequences
{consequences_text}

## Instructions
Analyze the player's action and determine:
1. Which consequences (if any) should TRIGGER based on this action
2. Which entities/NPCs might be relevant even if not explicitly mentioned

A consequence should trigger if the player's action matches or is closely related to its trigger condition. Be generous with semantic matching - "I enter the village" should trigger a consequence about "entering Riverside" if Riverside is a village.

Respond with ONLY a JSON object (no markdown, no explanation outside the JSON):
{{
  "triggered_consequences": ["id1", "id2"],
  "relevant_entities": ["Baron Aldric", "Town Guards"],
  "explanation": "Brief explanation of matches"
}}

If nothing is relevant, return empty arrays."#
        );

        // Make the API call
        let request = Request::new(vec![Message::user(&prompt)])
            .with_model(&self.model)
            .with_max_tokens(RELEVANCE_MAX_TOKENS)
            .with_temperature(0.0); // Deterministic for relevance checking

        let response = self.client.complete(request).await?;
        let response_text = response.text();

        // Parse the response
        self.parse_response(&response_text, story_memory)
    }

    /// Parse the model response into a RelevanceResult.
    fn parse_response(
        &self,
        response: &str,
        story_memory: &StoryMemory,
    ) -> Result<RelevanceResult, RelevanceError> {
        // Try to extract JSON from the response (handle potential markdown wrapping)
        let json_str = extract_json(response);

        // Parse the JSON
        let parsed: RelevanceResponse = serde_json::from_str(json_str)
            .map_err(|e| RelevanceError::ParseError(format!("{e}: {json_str}")))?;

        // Convert string IDs back to typed IDs
        let mut result = RelevanceResult {
            explanation: parsed.explanation,
            ..Default::default()
        };

        // Parse consequence IDs
        for id_str in parsed.triggered_consequences {
            // Try to find the consequence by ID string
            for consequence in story_memory.pending_consequences() {
                if consequence.id.to_string() == id_str {
                    result.triggered_consequences.push(consequence.id);
                    break;
                }
            }
        }

        // Parse entity names to IDs
        for name in parsed.relevant_entities {
            if let Some(id) = story_memory.find_entity_id(&name) {
                if !result.relevant_entities.contains(&id) {
                    result.relevant_entities.push(id);
                }
            }
        }

        Ok(result)
    }
}

// =============================================================================
// State Inference
// =============================================================================

/// An inferred state change detected from narrative text.
#[derive(Debug, Clone)]
pub struct InferredStateChange {
    /// The entity whose state changed.
    pub entity_name: String,
    /// What type of state changed.
    pub state_type: String,
    /// The inferred new value.
    pub new_value: String,
    /// Why this was inferred (quote from narrative).
    pub evidence: String,
    /// Confidence level (0.0 to 1.0).
    pub confidence: f32,
    /// Target entity for relationships.
    pub target_entity: Option<String>,
}

/// Response format for state inference.
#[derive(Debug, Deserialize)]
struct StateInferenceResponse {
    #[serde(default)]
    inferred_changes: Vec<InferredChange>,
}

#[derive(Debug, Deserialize)]
struct InferredChange {
    entity_name: String,
    state_type: String,
    new_value: String,
    evidence: String,
    confidence: f32,
    #[serde(default)]
    target_entity: Option<String>,
}

/// Infers state changes from DM narrative text.
///
/// This uses the configured fast model to detect when narrative implies state changes
/// that the DM didn't explicitly record with tools. For example:
/// - "Mira smiles warmly" → disposition changed to friendly
/// - "The guard captain storms off to the gate" → location changed
pub struct StateInferrer {
    client: Client,
    model: String,
}

impl StateInferrer {
    /// Create a new state inferrer with the given API client.
    pub fn new(client: Client) -> Self {
        let model = client.fast_model().to_string();
        Self { client, model }
    }

    /// Set a custom model for state inference.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Infer state changes from a DM narrative response.
    ///
    /// Returns changes that should be applied, filtered by confidence threshold.
    pub async fn infer_state_changes(
        &self,
        narrative: &str,
        known_entities: &[String],
        confidence_threshold: f32,
    ) -> Result<Vec<InferredStateChange>, RelevanceError> {
        // Skip if narrative is too short
        if narrative.len() < 20 {
            return Ok(Vec::new());
        }

        // Skip if no entities to track
        if known_entities.is_empty() {
            return Ok(Vec::new());
        }

        let entities_list = known_entities.join(", ");

        let prompt = format!(
            r#"Analyze this D&D narrative for implied state changes that weren't explicitly recorded.

## Narrative
"{narrative}"

## Known Entities
{entities_list}

## Instructions
Look for IMPLIED state changes in the narrative:
- Disposition: attitude changes (smiles, glares, thanks warmly, becomes hostile)
- Location: movement (storms off, follows, arrives at)
- Status: condition changes (injured, recovered, disappeared)
- Relationship: connection changes (befriends, betrays, owes a debt to)

Only report changes with high confidence (>0.7). Require explicit evidence in the text.

Respond with ONLY valid JSON (no markdown, no code blocks):
{{
  "inferred_changes": [
    {{
      "entity_name": "Mira",
      "state_type": "disposition",
      "new_value": "friendly",
      "evidence": "She smiles warmly and thanks you for your help",
      "confidence": 0.9,
      "target_entity": null
    }}
  ]
}}

IMPORTANT: Each field must be a single JSON value. The "evidence" field must be ONE quoted string containing all evidence text (not multiple comma-separated strings).

If no state changes are implied, return: {{"inferred_changes": []}}"#
        );

        let request = Request::new(vec![Message::user(&prompt)])
            .with_model(&self.model)
            .with_max_tokens(500)
            .with_temperature(0.0);

        let response = self.client.complete(request).await?;
        let response_text = response.text();

        // Parse response
        let json_str = extract_json(&response_text);
        // Try to fix common malformations (e.g., comma-separated strings for evidence)
        let sanitized = sanitize_json(json_str);
        let parsed: StateInferenceResponse = serde_json::from_str(&sanitized)
            .map_err(|e| RelevanceError::ParseError(format!("{e}: {sanitized}")))?;

        // Filter by confidence and convert
        let changes: Vec<InferredStateChange> = parsed
            .inferred_changes
            .into_iter()
            .filter(|c| c.confidence >= confidence_threshold)
            .map(|c| InferredStateChange {
                entity_name: c.entity_name,
                state_type: c.state_type,
                new_value: c.new_value,
                evidence: c.evidence,
                confidence: c.confidence,
                target_entity: c.target_entity,
            })
            .collect();

        Ok(changes)
    }
}

/// Extract JSON from a response that might have markdown code blocks or trailing text.
fn extract_json(text: &str) -> &str {
    let text = text.trim();

    // Handle ```json ... ``` blocks
    if let Some(start) = text.find("```json") {
        let content_start = start + 7;
        if let Some(end) = text[content_start..].find("```") {
            return text[content_start..content_start + end].trim();
        }
    }

    // Handle ``` ... ``` blocks (without json specifier)
    if let Some(start) = text.find("```") {
        let content_start = start + 3;
        if let Some(end) = text[content_start..].find("```") {
            return text[content_start..content_start + end].trim();
        }
    }

    // Handle JSON with trailing text (e.g., "{"inferred_changes": []}\n\nExplanation: ...")
    // Find the first '{' and match braces to find the complete JSON object
    if let Some(start) = text.find('{') {
        let bytes = text.as_bytes();
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, &byte) in bytes[start..].iter().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match byte {
                b'\\' if in_string => escape_next = true,
                b'"' => in_string = !in_string,
                b'{' if !in_string => depth += 1,
                b'}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        // Found the end of the JSON object
                        return &text[start..=start + i];
                    }
                }
                _ => {}
            }
        }
    }

    // Just return the text as-is
    text
}

/// Try to fix common JSON malformations from AI output.
///
/// Handles cases like:
/// `"evidence": "foo", "bar", "baz",` → `"evidence": "foo; bar; baz",`
fn sanitize_json(text: &str) -> String {
    // Look for the pattern: "evidence": "...", "...", "...",
    // where multiple quoted strings appear after the evidence key
    let evidence_key = r#""evidence":"#;

    if let Some(start_idx) = text.find(evidence_key) {
        let after_key = &text[start_idx + evidence_key.len()..];
        let after_key_trimmed = after_key.trim_start();

        // Should start with a quote
        if !after_key_trimmed.starts_with('"') {
            return text.to_string();
        }

        // Collect all quoted strings until we hit a non-string value or valid key
        let mut strings = Vec::new();
        let mut remaining = after_key_trimmed;

        loop {
            if !remaining.starts_with('"') {
                break;
            }

            // Find the end of this string (handle escaped quotes)
            let content_start = 1;
            let mut i = content_start;
            let chars: Vec<char> = remaining.chars().collect();

            while i < chars.len() {
                if chars[i] == '"' && (i == 0 || chars[i - 1] != '\\') {
                    break;
                }
                i += 1;
            }

            if i >= chars.len() {
                break; // Malformed, give up
            }

            let string_content: String = chars[content_start..i].iter().collect();
            strings.push(string_content);

            // Move past the closing quote
            remaining = &remaining[i + 1..];
            remaining = remaining.trim_start();

            // Check if there's a comma followed by another string
            if remaining.starts_with(',') {
                remaining = remaining[1..].trim_start();
                // If next char is a quote, continue; otherwise we're done
                if !remaining.starts_with('"') {
                    break;
                }
                // Check if this is a key (has colon after the string)
                if let Some(quote_end) = remaining[1..].find('"') {
                    let after_string = remaining[quote_end + 2..].trim_start();
                    if after_string.starts_with(':') {
                        // This is a key, not a continuation
                        break;
                    }
                }
            } else {
                break;
            }
        }

        // If we found multiple strings, combine them
        if strings.len() > 1 {
            let combined = strings.join("; ");
            let before = &text[..start_idx + evidence_key.len()];
            let fixed = format!(r#" "{}","#, combined);
            return format!("{}{}{}", before, fixed, remaining);
        }
    }

    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_plain() {
        let text = r#"{"triggered_consequences": [], "relevant_entities": []}"#;
        assert_eq!(extract_json(text), text);
    }

    #[test]
    fn test_extract_json_markdown() {
        let text = r#"```json
{"triggered_consequences": ["abc"], "relevant_entities": ["Guard"]}
```"#;
        let expected = r#"{"triggered_consequences": ["abc"], "relevant_entities": ["Guard"]}"#;
        assert_eq!(extract_json(text), expected);
    }

    #[test]
    fn test_extract_json_markdown_no_specifier() {
        let text = r#"```
{"triggered_consequences": []}
```"#;
        let expected = r#"{"triggered_consequences": []}"#;
        assert_eq!(extract_json(text), expected);
    }

    #[test]
    fn test_extract_json_with_whitespace() {
        let text = r#"
  {"triggered_consequences": []}
  "#;
        let result = extract_json(text);
        assert!(result.starts_with('{'));
        assert!(result.ends_with('}'));
    }

    #[test]
    fn test_extract_json_nested_backticks() {
        // If there's text before the code block
        let text = r#"Here is the JSON:
```json
{"triggered_consequences": ["id1"]}
```"#;
        let expected = r#"{"triggered_consequences": ["id1"]}"#;
        assert_eq!(extract_json(text), expected);
    }

    #[test]
    fn test_extract_json_with_trailing_explanation() {
        // JSON followed by explanation text (no markdown)
        let text = r#"{"inferred_changes": []}

Explanation: After carefully analyzing the narrative, I found no implied state changes."#;
        let expected = r#"{"inferred_changes": []}"#;
        assert_eq!(extract_json(text), expected);
    }

    #[test]
    fn test_extract_json_with_nested_braces() {
        // JSON with nested objects followed by trailing text
        let text = r#"{"inferred_changes": [{"entity_name": "Test", "data": {"nested": true}}]}

Some trailing explanation here."#;
        let expected =
            r#"{"inferred_changes": [{"entity_name": "Test", "data": {"nested": true}}]}"#;
        assert_eq!(extract_json(text), expected);
    }

    #[test]
    fn test_extract_json_with_braces_in_strings() {
        // JSON with braces inside string values
        let text = r#"{"evidence": "The guard said {hello} to you"}

Explanation: ..."#;
        let expected = r#"{"evidence": "The guard said {hello} to you"}"#;
        assert_eq!(extract_json(text), expected);
    }

    #[test]
    fn test_relevance_result_empty() {
        let result = RelevanceResult::default();
        assert!(result.is_empty());
        assert!(!result.has_triggered_consequences());
        assert!(!result.has_relevant_context());
    }

    #[test]
    fn test_relevance_result_with_consequences() {
        let result = RelevanceResult {
            triggered_consequences: vec![ConsequenceId::new()],
            relevant_facts: vec![],
            relevant_entities: vec![],
            explanation: None,
        };
        assert!(!result.is_empty());
        assert!(result.has_triggered_consequences());
        assert!(!result.has_relevant_context());
    }

    #[test]
    fn test_relevance_result_with_facts() {
        let result = RelevanceResult {
            triggered_consequences: vec![],
            relevant_facts: vec![FactId::new()],
            relevant_entities: vec![],
            explanation: None,
        };
        assert!(!result.is_empty());
        assert!(!result.has_triggered_consequences());
        assert!(result.has_relevant_context());
    }

    #[test]
    fn test_relevance_result_with_entities() {
        let result = RelevanceResult {
            triggered_consequences: vec![],
            relevant_facts: vec![],
            relevant_entities: vec![EntityId::new()],
            explanation: None,
        };
        assert!(!result.is_empty());
        assert!(!result.has_triggered_consequences());
        assert!(result.has_relevant_context());
    }

    #[test]
    fn test_relevance_result_with_explanation() {
        let result = RelevanceResult {
            triggered_consequences: vec![],
            relevant_facts: vec![],
            relevant_entities: vec![],
            explanation: Some("Test explanation".to_string()),
        };
        // Having only an explanation doesn't make it non-empty
        assert!(result.is_empty());
        assert_eq!(result.explanation, Some("Test explanation".to_string()));
    }

    #[test]
    fn test_relevance_response_parsing() {
        // Test that RelevanceResponse can be deserialized
        let json = r#"{"triggered_consequences": ["id1", "id2"], "relevant_entities": ["Guard", "King"], "explanation": "Test"}"#;
        let response: RelevanceResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.triggered_consequences.len(), 2);
        assert_eq!(response.relevant_entities.len(), 2);
        assert_eq!(response.explanation, Some("Test".to_string()));
    }

    #[test]
    fn test_relevance_response_defaults() {
        // Test that missing fields default correctly
        let json = r#"{}"#;
        let response: RelevanceResponse = serde_json::from_str(json).unwrap();
        assert!(response.triggered_consequences.is_empty());
        assert!(response.relevant_entities.is_empty());
        assert!(response.explanation.is_none());
    }

    #[test]
    fn test_sanitize_json_malformed_evidence() {
        // Test the exact error pattern from the bug report
        let malformed = r#"{
  "inferred_changes": [
    {
      "entity_name": "Protagonist",
      "state_type": "disposition",
      "new_value": "enraged",
      "evidence": "rage-fueled charge", "rage burns bright", "seeking a target for your fury",
      "confidence": 0.9,
      "target_entity": null
    }
  ]
}"#;
        let sanitized = sanitize_json(malformed);

        // The sanitized version should be valid JSON
        let parsed: Result<StateInferenceResponse, _> = serde_json::from_str(&sanitized);
        assert!(parsed.is_ok(), "Sanitized JSON should parse: {}", sanitized);

        // And the evidence should be combined
        let parsed = parsed.unwrap();
        assert_eq!(parsed.inferred_changes.len(), 1);
        assert!(parsed.inferred_changes[0]
            .evidence
            .contains("rage-fueled charge"));
        assert!(parsed.inferred_changes[0]
            .evidence
            .contains("rage burns bright"));
    }

    #[test]
    fn test_sanitize_json_valid_unchanged() {
        // Valid JSON should pass through unchanged
        let valid = r#"{"inferred_changes": [{"entity_name": "Test", "state_type": "status", "new_value": "ok", "evidence": "single string", "confidence": 0.8, "target_entity": null}]}"#;
        let sanitized = sanitize_json(valid);

        // Should parse the same
        let parsed: StateInferenceResponse = serde_json::from_str(&sanitized).unwrap();
        assert_eq!(parsed.inferred_changes[0].evidence, "single string");
    }
}
