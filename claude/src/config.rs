//! Provider and model configuration.

use std::fmt;

use crate::Error;

pub const ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";
pub const ANTHROPIC_DEFAULT_MODEL: &str = "claude-sonnet-4-6";
pub const ANTHROPIC_DEFAULT_FAST_MODEL: &str = "claude-haiku-4-5-20251001";

pub const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const OPENAI_DEFAULT_MODEL: &str = "gpt-5.6-sol";
pub const OPENAI_DEFAULT_FAST_MODEL: &str = "gpt-5.6-luna";

pub const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
pub const OPENROUTER_DEFAULT_MODEL: &str = "moonshotai/kimi-k3";

pub const OLLAMA_BASE_URL: &str = "http://localhost:11434/v1";
pub const LOCAL_DEFAULT_MODEL: &str = "qwen3.6:35b-a3b";

/// The wire protocol and credential namespace used by a client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Anthropic,
    OpenAi,
    OpenRouter,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenAi => "openai",
            Self::OpenRouter => "openrouter",
        }
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Complete configuration for one LLM endpoint.
///
/// API keys are intentionally private and this type does not implement
/// [`Debug`], preventing accidental credential logging.
#[derive(Clone)]
pub struct ClientConfig {
    pub(crate) provider: ProviderKind,
    pub(crate) api_key: String,
    pub(crate) base_url: String,
    pub(crate) model: String,
    pub(crate) fast_model: String,
}

impl ClientConfig {
    pub fn anthropic(api_key: impl Into<String>) -> Self {
        Self::new(
            ProviderKind::Anthropic,
            api_key,
            ANTHROPIC_BASE_URL,
            ANTHROPIC_DEFAULT_MODEL,
            ANTHROPIC_DEFAULT_FAST_MODEL,
        )
    }

    pub fn openai(api_key: impl Into<String>) -> Self {
        Self::new(
            ProviderKind::OpenAi,
            api_key,
            OPENAI_BASE_URL,
            OPENAI_DEFAULT_MODEL,
            OPENAI_DEFAULT_FAST_MODEL,
        )
    }

    pub fn openrouter(api_key: impl Into<String>) -> Self {
        Self::new(
            ProviderKind::OpenRouter,
            api_key,
            OPENROUTER_BASE_URL,
            OPENROUTER_DEFAULT_MODEL,
            OPENROUTER_DEFAULT_MODEL,
        )
    }

    /// Configure an OpenAI-compatible Ollama endpoint.
    pub fn local(model: impl Into<String>) -> Self {
        let model = model.into();
        Self::new(
            ProviderKind::OpenAi,
            "unused",
            OLLAMA_BASE_URL,
            model.clone(),
            model,
        )
    }

    fn new(
        provider: ProviderKind,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
        fast_model: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            api_key: api_key.into(),
            base_url: normalize_base_url(base_url.into()),
            model: model.into(),
            fast_model: fast_model.into(),
        }
    }

    /// Resolve a provider from environment variables.
    ///
    /// `CHRONICLER_PROVIDER` may be `anthropic`, `openai`, `openrouter`, or
    /// `local`. Without it, OpenRouter is preferred when configured, followed
    /// by Anthropic, then OpenAI/OpenAI-compatible.
    pub fn from_env() -> Result<Self, Error> {
        let selected = nonempty_env("CHRONICLER_PROVIDER").map(|value| value.to_lowercase());
        let provider = match selected.as_deref() {
            Some("anthropic") => "anthropic",
            Some("openai") => "openai",
            Some("openrouter") => "openrouter",
            Some("local") | Some("ollama") => "local",
            Some(other) => {
                return Err(Error::Config(format!(
                    "unknown CHRONICLER_PROVIDER '{other}' (expected anthropic, openai, openrouter, or local)"
                )))
            }
            None if nonempty_env("OPENROUTER_API_KEY").is_some() => "openrouter",
            None if nonempty_env("ANTHROPIC_API_KEY").is_some() => "anthropic",
            None if nonempty_env("OPENAI_API_KEY").is_some()
                || nonempty_env("OPENAI_BASE_URL").is_some() =>
            {
                "openai"
            }
            None => return Err(Error::NoApiKey),
        };

        let mut config = match provider {
            "anthropic" => {
                let key = required_env("ANTHROPIC_API_KEY")?;
                Self::anthropic(key).with_base_url(
                    nonempty_env("ANTHROPIC_BASE_URL")
                        .unwrap_or_else(|| ANTHROPIC_BASE_URL.to_string()),
                )
            }
            "openrouter" => {
                let key = required_env("OPENROUTER_API_KEY")?;
                Self::openrouter(key).with_base_url(
                    nonempty_env("OPENROUTER_BASE_URL")
                        .unwrap_or_else(|| OPENROUTER_BASE_URL.to_string()),
                )
            }
            "local" => {
                let model = model_override("OLLAMA_MODEL", LOCAL_DEFAULT_MODEL);
                Self::local(model).with_base_url(
                    nonempty_env("OLLAMA_BASE_URL")
                        .or_else(|| nonempty_env("OPENAI_BASE_URL"))
                        .unwrap_or_else(|| OLLAMA_BASE_URL.to_string()),
                )
            }
            "openai" => {
                let base_url =
                    nonempty_env("OPENAI_BASE_URL").unwrap_or_else(|| OPENAI_BASE_URL.to_string());
                let key = match nonempty_env("OPENAI_API_KEY") {
                    Some(key) => key,
                    None if normalize_base_url(base_url.clone()) != OPENAI_BASE_URL => {
                        "unused".to_string()
                    }
                    None => return Err(missing_key("OPENAI_API_KEY")),
                };
                Self::openai(key).with_base_url(base_url)
            }
            _ => unreachable!(),
        };

        let provider_model_var = match config.provider {
            ProviderKind::Anthropic => "ANTHROPIC_MODEL",
            ProviderKind::OpenAi if provider == "local" => "OLLAMA_MODEL",
            ProviderKind::OpenAi => "OPENAI_MODEL",
            ProviderKind::OpenRouter => "OPENROUTER_MODEL",
        };
        if let Some(model) =
            nonempty_env("CHRONICLER_MODEL").or_else(|| nonempty_env(provider_model_var))
        {
            config.model = model;
        }

        let provider_fast_model_var = match config.provider {
            ProviderKind::Anthropic => "ANTHROPIC_FAST_MODEL",
            ProviderKind::OpenAi if provider == "local" => "OLLAMA_FAST_MODEL",
            ProviderKind::OpenAi => "OPENAI_FAST_MODEL",
            ProviderKind::OpenRouter => "OPENROUTER_FAST_MODEL",
        };
        if let Some(model) =
            nonempty_env("CHRONICLER_FAST_MODEL").or_else(|| nonempty_env(provider_fast_model_var))
        {
            config.fast_model = model;
        }

        Ok(config)
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = normalize_base_url(base_url.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_fast_model(mut self, model: impl Into<String>) -> Self {
        self.fast_model = model.into();
        self
    }

    pub fn provider(&self) -> ProviderKind {
        self.provider
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn fast_model(&self) -> &str {
        &self.fast_model
    }
}

fn nonempty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn required_env(name: &str) -> Result<String, Error> {
    nonempty_env(name).ok_or_else(|| missing_key(name))
}

fn missing_key(name: &str) -> Error {
    Error::Config(format!("{name} is not set"))
}

fn model_override(provider_variable: &str, default: &str) -> String {
    nonempty_env("CHRONICLER_MODEL")
        .or_else(|| nonempty_env(provider_variable))
        .unwrap_or_else(|| default.to_string())
}

fn normalize_base_url(base_url: String) -> String {
    base_url.trim().trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_defaults_match_generalist() {
        let config = ClientConfig::local(LOCAL_DEFAULT_MODEL);
        assert_eq!(config.provider(), ProviderKind::OpenAi);
        assert_eq!(config.base_url(), OLLAMA_BASE_URL);
        assert_eq!(config.model(), "qwen3.6:35b-a3b");
        assert_eq!(config.fast_model(), config.model());
    }

    #[test]
    fn trims_trailing_slashes() {
        let config =
            ClientConfig::openrouter("secret").with_base_url("https://openrouter.ai/api/v1///");
        assert_eq!(config.base_url(), OPENROUTER_BASE_URL);
    }
}
