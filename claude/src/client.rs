//! Provider-neutral LLM client implementation.

use std::pin::Pin;

use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use tokio_stream::Stream;

use crate::api_types::{
    ApiContent, ApiContentBlock, ApiMessage, ApiRequest, ApiResponse, ApiTool, ApiToolChoice,
};
use crate::config::{ClientConfig, ProviderKind};
use crate::error::Error;
use crate::openai;
use crate::streaming::parse_sse_events_buffered;
use crate::types::{
    ContentBlock, Message, Request, Response, Role, StopReason, StreamEvent, ToolChoice,
    ToolResult, ToolUse, Usage,
};

const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Client for Anthropic Messages and OpenAI-compatible Chat Completions APIs.
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    config: ClientConfig,
}

impl Client {
    /// Build a client from explicit provider configuration.
    pub fn from_config(config: ClientConfig) -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(600))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|error| Error::Config(format!("failed to build HTTP client: {error}")))?;
        Ok(Self { http, config })
    }

    /// Resolve provider, credentials, endpoint, and models from the environment.
    pub fn from_env() -> Result<Self, Error> {
        Self::from_config(ClientConfig::from_env()?)
    }

    pub fn anthropic(api_key: impl Into<String>) -> Result<Self, Error> {
        Self::from_config(ClientConfig::anthropic(api_key))
    }

    pub fn openai(api_key: impl Into<String>) -> Result<Self, Error> {
        Self::from_config(ClientConfig::openai(api_key))
    }

    pub fn openrouter(api_key: impl Into<String>) -> Result<Self, Error> {
        Self::from_config(ClientConfig::openrouter(api_key))
    }

    /// Connect to Ollama's local OpenAI-compatible endpoint.
    pub fn local(model: impl Into<String>) -> Result<Self, Error> {
        Self::from_config(ClientConfig::local(model))
    }

    /// Override the default narrative model.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Override the cheaper/faster model used by background tasks.
    pub fn with_fast_model(mut self, model: impl Into<String>) -> Self {
        self.config.fast_model = model.into();
        self
    }

    pub fn provider(&self) -> ProviderKind {
        self.config.provider
    }

    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }

    pub fn fast_model(&self) -> &str {
        &self.config.fast_model
    }

    /// Send a non-streaming completion request.
    pub async fn complete(&self, request: Request) -> Result<Response, Error> {
        match self.config.provider {
            ProviderKind::Anthropic => self.complete_anthropic(request).await,
            ProviderKind::OpenAi | ProviderKind::OpenRouter => {
                openai::complete(&self.http, &self.config, &request).await
            }
        }
    }

    /// Send a streaming completion request.
    pub async fn stream(
        &self,
        request: Request,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, Error>> + Send>>, Error> {
        match self.config.provider {
            ProviderKind::Anthropic => self.stream_anthropic(request).await,
            ProviderKind::OpenAi | ProviderKind::OpenRouter => {
                openai::stream(&self.http, &self.config, &request).await
            }
        }
    }

    /// Run a tool-use loop until the model completes without requesting tools.
    pub async fn complete_with_tools<F, Fut>(
        &self,
        mut request: Request,
        mut executor: F,
    ) -> Result<Response, Error>
    where
        F: FnMut(ToolUse) -> Fut,
        Fut: std::future::Future<Output = ToolResult>,
    {
        loop {
            let response = self.complete(request.clone()).await?;
            if response.stop_reason != StopReason::ToolUse {
                return Ok(response);
            }

            let tool_uses: Vec<ToolUse> = response
                .content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::ToolUse { id, name, input } => Some(ToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                    }),
                    _ => None,
                })
                .collect();
            if tool_uses.is_empty() {
                return Ok(response);
            }

            request.messages.push(Message {
                role: Role::Assistant,
                content: response.content.clone(),
            });

            let mut tool_results = Vec::new();
            for tool_use in tool_uses {
                let result = executor(tool_use.clone()).await;
                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: tool_use.id,
                    content: result.content,
                    is_error: result.is_error,
                });
            }
            request.messages.push(Message {
                role: Role::User,
                content: tool_results,
            });
        }
    }

    async fn complete_anthropic(&self, request: Request) -> Result<Response, Error> {
        let response = self
            .http
            .post(format!("{}/messages", self.config.base_url))
            .headers(self.anthropic_headers()?)
            .json(&self.build_anthropic_request(&request, false))
            .send()
            .await
            .map_err(|error| Error::Network(error.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::Api { status, message });
        }

        let response: ApiResponse = response
            .json()
            .await
            .map_err(|error| Error::Parse(error.to_string()))?;
        Ok(parse_anthropic_response(response))
    }

    async fn stream_anthropic(
        &self,
        request: Request,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, Error>> + Send>>, Error> {
        let response = self
            .http
            .post(format!("{}/messages", self.config.base_url))
            .headers(self.anthropic_headers()?)
            .json(&self.build_anthropic_request(&request, true))
            .send()
            .await
            .map_err(|error| Error::Network(error.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::Api { status, message });
        }

        let stream = response
            .bytes_stream()
            .scan(String::new(), |buffer, result| {
                let events = match result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        parse_sse_events_buffered(buffer)
                    }
                    Err(error) => vec![Err(Error::Network(error.to_string()))],
                };
                futures::future::ready(Some(events))
            })
            .flat_map(futures::stream::iter);
        Ok(Box::pin(stream))
    }

    fn anthropic_headers(&self) -> Result<HeaderMap, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.config.api_key)
                .map_err(|error| Error::Config(format!("invalid API key: {error}")))?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(ANTHROPIC_API_VERSION),
        );
        Ok(headers)
    }

    fn build_anthropic_request(&self, request: &Request, stream: bool) -> ApiRequest {
        let messages = request
            .messages
            .iter()
            .map(|message| ApiMessage {
                role: match message.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: message
                    .content
                    .iter()
                    .filter_map(ApiContentBlock::from_content)
                    .collect(),
            })
            .collect();
        let tools = request.tools.as_ref().map(|tools| {
            tools
                .iter()
                .map(|tool| ApiTool {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    input_schema: tool.input_schema.clone(),
                })
                .collect()
        });

        ApiRequest {
            model: request
                .model
                .clone()
                .unwrap_or_else(|| self.config.model.clone()),
            max_tokens: request.max_tokens,
            system: request.system.clone(),
            messages,
            temperature: request.temperature,
            tools,
            tool_choice: request.tool_choice.as_ref().map(|choice| match choice {
                ToolChoice::Auto => ApiToolChoice {
                    r#type: "auto".to_string(),
                    name: None,
                },
                ToolChoice::Any => ApiToolChoice {
                    r#type: "any".to_string(),
                    name: None,
                },
                ToolChoice::Tool { name } => ApiToolChoice {
                    r#type: "tool".to_string(),
                    name: Some(name.clone()),
                },
            }),
            stream,
        }
    }
}

fn parse_anthropic_response(response: ApiResponse) -> Response {
    let content = response
        .content
        .into_iter()
        .map(|content| match content {
            ApiContent::Text { text } => ContentBlock::Text { text },
            ApiContent::ToolUse { id, name, input } => ContentBlock::ToolUse { id, name, input },
            ApiContent::Thinking { thinking } => ContentBlock::Thinking { thinking },
        })
        .collect();
    Response {
        id: response.id,
        model: response.model,
        content,
        stop_reason: match response.stop_reason.as_str() {
            "max_tokens" => StopReason::MaxTokens,
            "stop_sequence" => StopReason::StopSequence,
            "tool_use" => StopReason::ToolUse,
            _ => StopReason::EndTurn,
        },
        usage: Usage {
            input_tokens: response.usage.input_tokens,
            output_tokens: response.usage.output_tokens,
        },
    }
}
