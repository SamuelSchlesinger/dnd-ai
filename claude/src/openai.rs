//! OpenAI-compatible Chat Completions transport.
//!
//! This dialect is shared by OpenAI, OpenRouter, Ollama, and many local
//! inference servers. Provider-specific behavior is kept deliberately small:
//! official OpenAI GPT-5.6 requests disable reasoning for function-tool
//! compatibility, while OpenRouter reasoning metadata is replayed verbatim.

use std::collections::{BTreeMap, VecDeque};
use std::pin::Pin;

use futures::{Stream, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Map, Value};

use crate::config::{ClientConfig, ProviderKind, OPENAI_BASE_URL};
use crate::error::Error;
use crate::types::{
    ContentBlock, Message, Request, Response, Role, StopReason, StreamEvent, ToolChoice, Usage,
};

pub(crate) async fn complete(
    http: &reqwest::Client,
    config: &ClientConfig,
    request: &Request,
) -> Result<Response, Error> {
    let response = http
        .post(format!("{}/chat/completions", config.base_url))
        .headers(headers(config)?)
        .json(&build_body(config, request, false))
        .send()
        .await
        .map_err(|error| Error::Network(error.to_string()))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let message = response.text().await.unwrap_or_default();
        return Err(Error::Api { status, message });
    }

    let value: Value = response
        .json()
        .await
        .map_err(|error| Error::Parse(error.to_string()))?;
    parse_response(&value, request.model.as_deref().unwrap_or(&config.model))
}

pub(crate) async fn stream(
    http: &reqwest::Client,
    config: &ClientConfig,
    request: &Request,
) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, Error>> + Send>>, Error> {
    let response = http
        .post(format!("{}/chat/completions", config.base_url))
        .headers(headers(config)?)
        .json(&build_body(config, request, true))
        .send()
        .await
        .map_err(|error| Error::Network(error.to_string()))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let message = response.text().await.unwrap_or_default();
        return Err(Error::Api { status, message });
    }

    let source = Box::pin(response.bytes_stream());
    let stream = futures::stream::unfold(
        (
            source,
            OpenAiStreamState::default(),
            VecDeque::<Result<StreamEvent, Error>>::new(),
            false,
        ),
        |(mut source, mut state, mut pending, mut source_done)| async move {
            loop {
                if let Some(event) = pending.pop_front() {
                    return Some((event, (source, state, pending, source_done)));
                }
                if source_done {
                    return None;
                }

                match source.next().await {
                    Some(Ok(bytes)) => {
                        for data in state.sse.push(bytes.as_ref()) {
                            pending.extend(state.process_data(&data));
                        }
                    }
                    Some(Err(error)) => {
                        pending.push_back(Err(Error::Network(error.to_string())));
                        source_done = true;
                    }
                    None => {
                        if let Some(data) = state.sse.finish() {
                            pending.extend(state.process_data(&data));
                        }
                        pending.extend(state.finish());
                        source_done = true;
                    }
                }
            }
        },
    );

    Ok(Box::pin(stream))
}

fn headers(config: &ClientConfig) -> Result<HeaderMap, Error> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", config.api_key))
            .map_err(|error| Error::Config(format!("invalid API key: {error}")))?,
    );
    Ok(headers)
}

fn build_body(config: &ClientConfig, request: &Request, stream: bool) -> Value {
    let model = request.model.as_deref().unwrap_or(&config.model);
    let mut body = Map::new();
    body.insert("model".to_string(), Value::String(model.to_string()));
    body.insert(
        "messages".to_string(),
        Value::Array(to_wire_messages(config, request)),
    );
    body.insert("stream".to_string(), Value::Bool(stream));

    if is_official_openai(config) {
        body.insert(
            "max_completion_tokens".to_string(),
            Value::from(request.max_tokens),
        );
        // GPT-5.6 Chat Completions function tools require effective reasoning
        // `none`; making it explicit avoids the model's default medium effort.
        if model.starts_with("gpt-5.6") {
            body.insert(
                "reasoning_effort".to_string(),
                Value::String("none".to_string()),
            );
        }
    }

    if let Some(temperature) = request.temperature {
        if let Some(number) = serde_json::Number::from_f64(f64::from(temperature)) {
            body.insert("temperature".to_string(), Value::Number(number));
        }
    }

    if let Some(tools) = request.tools.as_ref().filter(|tools| !tools.is_empty()) {
        body.insert(
            "tools".to_string(),
            Value::Array(
                tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": tool.name,
                                "description": tool.description,
                                "parameters": tool.input_schema,
                            }
                        })
                    })
                    .collect(),
            ),
        );
    }

    if let Some(choice) = &request.tool_choice {
        body.insert(
            "tool_choice".to_string(),
            match choice {
                ToolChoice::Auto => Value::String("auto".to_string()),
                ToolChoice::Any => Value::String("required".to_string()),
                ToolChoice::Tool { name } => json!({
                    "type": "function",
                    "function": { "name": name }
                }),
            },
        );
    }

    Value::Object(body)
}

fn to_wire_messages(config: &ClientConfig, request: &Request) -> Vec<Value> {
    let mut output = Vec::new();
    if let Some(system) = request.system.as_ref().filter(|text| !text.is_empty()) {
        output.push(json!({"role": "system", "content": system}));
    }

    for message in &request.messages {
        match message.role {
            Role::Assistant => output.push(to_assistant_message(config, message)),
            Role::User => output.extend(to_user_messages(message)),
        }
    }
    output
}

fn to_assistant_message(config: &ClientConfig, message: &Message) -> Value {
    let text = message
        .content
        .iter()
        .filter_map(ContentBlock::as_text)
        .collect::<Vec<_>>()
        .join("");
    let tool_calls: Vec<Value> = message
        .content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::ToolUse { id, name, input } => Some(json!({
                "id": id,
                "type": "function",
                "function": { "name": name, "arguments": input.to_string() }
            })),
            _ => None,
        })
        .collect();

    let mut wire = Map::new();
    wire.insert("role".to_string(), Value::String("assistant".to_string()));
    wire.insert(
        "content".to_string(),
        if text.is_empty() {
            Value::Null
        } else {
            Value::String(text)
        },
    );
    if !tool_calls.is_empty() {
        wire.insert("tool_calls".to_string(), Value::Array(tool_calls));
    }

    if config.provider == ProviderKind::OpenRouter {
        let reasoning_details: Vec<Value> = message
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::ReasoningDetails { details } => Some(details.as_slice()),
                _ => None,
            })
            .flatten()
            .cloned()
            .collect();
        if !reasoning_details.is_empty() {
            wire.insert(
                "reasoning_details".to_string(),
                Value::Array(reasoning_details),
            );
        } else {
            let reasoning = message
                .content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Thinking { thinking } => Some(thinking.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("");
            if !reasoning.is_empty() {
                wire.insert("reasoning".to_string(), Value::String(reasoning));
            }
        }
    }

    Value::Object(wire)
}

fn to_user_messages(message: &Message) -> Vec<Value> {
    let mut output = Vec::new();
    for block in &message.content {
        if let ContentBlock::ToolResult {
            tool_use_id,
            content,
            is_error,
        } = block
        {
            let content = if *is_error {
                format!("Error: {content}")
            } else {
                content.clone()
            };
            output.push(json!({
                "role": "tool",
                "tool_call_id": tool_use_id,
                "content": content,
            }));
        }
    }

    let mut parts = Vec::new();
    for block in &message.content {
        match block {
            ContentBlock::Text { text } if !text.is_empty() => {
                parts.push(json!({"type": "text", "text": text}));
            }
            ContentBlock::Image { media_type, data } => {
                parts.push(json!({
                    "type": "image_url",
                    "image_url": {"url": format!("data:{media_type};base64,{data}")}
                }));
            }
            _ => {}
        }
    }
    if parts.len() == 1 && parts[0].get("type").and_then(Value::as_str) == Some("text") {
        output.push(json!({
            "role": "user",
            "content": parts[0].get("text").cloned().unwrap_or(Value::Null),
        }));
    } else if !parts.is_empty() {
        output.push(json!({"role": "user", "content": parts}));
    }
    output
}

fn parse_response(value: &Value, fallback_model: &str) -> Result<Response, Error> {
    let choice = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .ok_or_else(|| Error::Parse("response is missing choices[0]".to_string()))?;
    let message = choice
        .get("message")
        .ok_or_else(|| Error::Parse("response choice is missing message".to_string()))?;

    let mut content = Vec::new();
    if let Some(details) = message.get("reasoning_details").and_then(Value::as_array) {
        if !details.is_empty() {
            content.push(ContentBlock::ReasoningDetails {
                details: details.clone(),
            });
        }
    }
    if let Some(reasoning) = reasoning_text(message) {
        content.push(ContentBlock::Thinking {
            thinking: reasoning.to_string(),
        });
    }
    if let Some(text) = content_text(message.get("content")) {
        if !text.is_empty() {
            content.push(ContentBlock::Text { text });
        }
    }

    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        for (index, call) in calls.iter().enumerate() {
            let function = call.get("function").unwrap_or(&Value::Null);
            let id = call
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("call_{index}"));
            let name = function
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let input = match function.get("arguments") {
                Some(Value::String(arguments)) => serde_json::from_str(arguments)
                    .unwrap_or_else(|_| json!({"_unparsed_arguments": arguments})),
                Some(value @ Value::Object(_)) => value.clone(),
                _ => json!({}),
            };
            content.push(ContentBlock::ToolUse { id, name, input });
        }
    }

    let has_tools = content
        .iter()
        .any(|block| matches!(block, ContentBlock::ToolUse { .. }));
    let stop_reason = choice
        .get("finish_reason")
        .and_then(Value::as_str)
        .map(parse_stop_reason)
        .unwrap_or(if has_tools {
            StopReason::ToolUse
        } else {
            StopReason::EndTurn
        });
    let usage = value.get("usage").unwrap_or(&Value::Null);

    Ok(Response {
        id: value
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        model: value
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or(fallback_model)
            .to_string(),
        content,
        stop_reason,
        usage: Usage {
            input_tokens: usage
                .get("prompt_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize,
            output_tokens: usage
                .get("completion_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize,
        },
    })
}

fn content_text(content: Option<&Value>) -> Option<String> {
    match content? {
        Value::String(text) => Some(text.clone()),
        Value::Array(parts) => Some(
            parts
                .iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join(""),
        ),
        _ => None,
    }
}

fn reasoning_text(value: &Value) -> Option<&str> {
    ["reasoning_content", "reasoning", "thinking"]
        .into_iter()
        .find_map(|field| value.get(field).and_then(Value::as_str))
        .filter(|text| !text.is_empty())
}

fn is_official_openai(config: &ClientConfig) -> bool {
    config.provider == ProviderKind::OpenAi && config.base_url == OPENAI_BASE_URL
}

fn parse_stop_reason(reason: &str) -> StopReason {
    match reason {
        "length" | "max_tokens" => StopReason::MaxTokens,
        "tool_calls" | "function_call" | "tool_use" => StopReason::ToolUse,
        "stop_sequence" => StopReason::StopSequence,
        _ => StopReason::EndTurn,
    }
}

#[derive(Default)]
struct SseAssembler {
    buffer: String,
    data_lines: Vec<String>,
}

impl SseAssembler {
    fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.buffer.push_str(&String::from_utf8_lossy(chunk));
        let mut events = Vec::new();
        while let Some(newline) = self.buffer.find('\n') {
            let line: String = self.buffer.drain(..=newline).collect();
            let line = line.trim_end_matches(['\n', '\r']);
            if line.is_empty() {
                if !self.data_lines.is_empty() {
                    events.push(self.data_lines.join("\n"));
                    self.data_lines.clear();
                }
            } else if let Some(data) = line.strip_prefix("data:") {
                self.data_lines.push(data.trim_start().to_string());
            }
        }
        events
    }

    fn finish(&mut self) -> Option<String> {
        if let Some(data) = self
            .buffer
            .trim_end_matches(['\n', '\r'])
            .strip_prefix("data:")
        {
            self.data_lines.push(data.trim_start().to_string());
        }
        self.buffer.clear();
        if self.data_lines.is_empty() {
            None
        } else {
            Some(self.data_lines.drain(..).collect::<Vec<_>>().join("\n"))
        }
    }
}

#[derive(Default)]
struct PartialToolCall {
    id: String,
    name: String,
    arguments: String,
}

#[derive(Default)]
struct OpenAiStreamState {
    sse: SseAssembler,
    started: bool,
    tools: BTreeMap<usize, PartialToolCall>,
    tools_emitted: bool,
    stop_emitted: bool,
    message_stopped: bool,
}

impl OpenAiStreamState {
    fn process_data(&mut self, data: &str) -> Vec<Result<StreamEvent, Error>> {
        if data == "[DONE]" {
            return self.finish();
        }

        let chunk: Value = match serde_json::from_str(data) {
            Ok(chunk) => chunk,
            Err(error) => {
                return vec![Err(Error::Parse(format!(
                    "chat-completions SSE parse error: {error}"
                )))]
            }
        };
        let mut events = Vec::new();

        if !self.started {
            self.started = true;
            events.push(Ok(StreamEvent::MessageStart {
                id: chunk
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                model: chunk
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            }));
        }

        let Some(choice) = chunk
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
        else {
            return events;
        };
        let delta = choice.get("delta").unwrap_or(&Value::Null);

        if let Some(details) = delta.get("reasoning_details").and_then(Value::as_array) {
            if !details.is_empty() {
                events.push(Ok(StreamEvent::ReasoningDetails {
                    details: details.clone(),
                }));
            }
        }
        if let Some(reasoning) = reasoning_text(delta) {
            events.push(Ok(StreamEvent::ThinkingDelta {
                index: 0,
                text: reasoning.to_string(),
            }));
        }
        if let Some(text) = delta
            .get("content")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
        {
            events.push(Ok(StreamEvent::TextDelta {
                index: 0,
                text: text.to_string(),
            }));
        }
        if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                let index = call.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                let partial = self.tools.entry(index).or_default();
                if let Some(id) = call.get("id").and_then(Value::as_str) {
                    partial.id.push_str(id);
                }
                if let Some(name) = call.pointer("/function/name").and_then(Value::as_str) {
                    partial.name.push_str(name);
                }
                if let Some(arguments) = call.pointer("/function/arguments") {
                    match arguments {
                        Value::String(fragment) => partial.arguments.push_str(fragment),
                        Value::Object(_) => partial.arguments.push_str(&arguments.to_string()),
                        _ => {}
                    }
                }
            }
        }

        if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
            events.extend(self.emit_tools());
            events.push(Ok(StreamEvent::MessageDelta {
                stop_reason: Some(parse_stop_reason(reason)),
            }));
            self.stop_emitted = true;
        }
        events
    }

    fn emit_tools(&mut self) -> Vec<Result<StreamEvent, Error>> {
        if self.tools_emitted {
            return Vec::new();
        }
        self.tools_emitted = true;
        let mut events = Vec::new();
        for (wire_index, tool) in &self.tools {
            let index = wire_index + 1;
            events.push(Ok(StreamEvent::ContentBlockStart {
                index,
                content_type: "tool_use".to_string(),
                tool_use_id: Some(if tool.id.is_empty() {
                    format!("call_{wire_index}")
                } else {
                    tool.id.clone()
                }),
                tool_name: Some(tool.name.clone()),
            }));
            events.push(Ok(StreamEvent::InputJsonDelta {
                index,
                partial_json: if tool.arguments.is_empty() {
                    "{}".to_string()
                } else {
                    tool.arguments.clone()
                },
            }));
            events.push(Ok(StreamEvent::ContentBlockStop { index }));
        }
        events
    }

    fn finish(&mut self) -> Vec<Result<StreamEvent, Error>> {
        if self.message_stopped {
            return Vec::new();
        }
        let mut events = self.emit_tools();
        if !self.stop_emitted {
            events.push(Ok(StreamEvent::MessageDelta {
                stop_reason: Some(if self.tools.is_empty() {
                    StopReason::EndTurn
                } else {
                    StopReason::ToolUse
                }),
            }));
            self.stop_emitted = true;
        }
        events.push(Ok(StreamEvent::MessageStop));
        self.message_stopped = true;
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Tool, ToolChoice};

    #[test]
    fn openrouter_replays_reasoning_details_and_tool_calls() {
        let config = ClientConfig::openrouter("secret");
        let details = vec![json!({"type": "reasoning.text", "text": "opaque"})];
        let request = Request::new(vec![Message {
            role: Role::Assistant,
            content: vec![
                ContentBlock::ReasoningDetails {
                    details: details.clone(),
                },
                ContentBlock::ToolUse {
                    id: "call_1".to_string(),
                    name: "roll".to_string(),
                    input: json!({"sides": 20}),
                },
            ],
        }]);
        let body = build_body(&config, &request, false);
        assert_eq!(body["messages"][0]["reasoning_details"], json!(details));
        assert_eq!(
            body["messages"][0]["tool_calls"][0]["function"]["name"],
            "roll"
        );
    }

    #[test]
    fn openai_tools_use_chat_completions_shape() {
        let config = ClientConfig::openai("secret");
        let request = Request::new(vec![Message::user("roll")])
            .with_tools(vec![Tool {
                name: "roll".to_string(),
                description: "Roll a die".to_string(),
                input_schema: json!({"type": "object"}),
            }])
            .with_tool_choice(ToolChoice::Any);
        let body = build_body(&config, &request, false);
        assert_eq!(body["tool_choice"], "required");
        assert_eq!(body["reasoning_effort"], "none");
        assert!(body.get("max_completion_tokens").is_some());
        assert!(body.get("max_tokens").is_none());
    }

    #[test]
    fn compatible_endpoints_do_not_cap_reasoning_before_tool_calls() {
        let config = ClientConfig::local("qwen3.6:35b-a3b");
        let body = build_body(&config, &Request::new(vec![Message::user("roll")]), false);
        assert!(body.get("max_tokens").is_none());
        assert!(body.get("max_completion_tokens").is_none());
    }

    #[test]
    fn parses_kimi_response_without_exposing_reasoning_as_text() {
        let value = json!({
            "id": "generation-1",
            "model": "moonshotai/kimi-k3",
            "choices": [{
                "finish_reason": "tool_calls",
                "message": {
                    "role": "assistant",
                    "content": null,
                    "reasoning": "private chain",
                    "reasoning_details": [{"type": "reasoning.text", "text": "opaque"}],
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {"name": "roll", "arguments": "{\"sides\":20}"}
                    }]
                }
            }],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5}
        });
        let response = parse_response(&value, "fallback").expect("response");
        assert_eq!(response.stop_reason, StopReason::ToolUse);
        assert_eq!(response.text(), "");
        assert!(matches!(
            response.content.first(),
            Some(ContentBlock::ReasoningDetails { .. })
        ));
        assert!(response
            .content
            .iter()
            .any(|block| matches!(block, ContentBlock::ToolUse { name, .. } if name == "roll")));
    }

    #[test]
    fn stream_accumulates_parallel_tool_calls_by_index() {
        let mut state = OpenAiStreamState::default();
        state.process_data(
            &json!({
                "id": "1", "model": "test",
                "choices": [{"delta": {"tool_calls": [
                    {"index": 0, "id": "a", "function": {"name": "one", "arguments": "{\"x\":"}},
                    {"index": 1, "id": "b", "function": {"name": "two", "arguments": "{}"}}
                ]}, "finish_reason": null}]
            })
            .to_string(),
        );
        let events = state.process_data(
            &json!({
                "choices": [{"delta": {"tool_calls": [
                    {"index": 0, "function": {"arguments": "1}"}}
                ]}, "finish_reason": "tool_calls"}]
            })
            .to_string(),
        );
        let tool_starts = events
            .iter()
            .filter(|event| matches!(event, Ok(StreamEvent::ContentBlockStart { .. })))
            .count();
        assert_eq!(tool_starts, 2);
        assert_eq!(state.tools[&0].arguments, "{\"x\":1}");
    }
}
