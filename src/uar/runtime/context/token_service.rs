use crate::llm::Message;
use tiktoken_rs::{CoreBPE, bpe_for_model, cl100k_base_singleton, o200k_base_singleton};

/// Which tiktoken encoding a model resolves to.
///
/// `Cl100kFallback` is distinct from `Cl100kBase`: both count with the same
/// encoding, but the fallback means the model was not recognized, so a caller
/// can label the count as an approximation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenEncoding {
    /// The model's tokenizer is `o200k_base`.
    O200kBase,
    /// The model's tokenizer is `cl100k_base`.
    Cl100kBase,
    /// The model is unrecognized; `cl100k_base` is the documented fallback.
    Cl100kFallback,
}

impl TokenEncoding {
    fn bpe(self) -> &'static CoreBPE {
        match self {
            Self::O200kBase => o200k_base_singleton(),
            Self::Cl100kBase | Self::Cl100kFallback => cl100k_base_singleton(),
        }
    }

    /// True when the count is an approximation rather than the model's own
    /// tokenizer.
    #[must_use]
    pub fn is_fallback(self) -> bool {
        matches!(self, Self::Cl100kFallback)
    }
}

/// Per-message protocol overhead in the OpenAI chat format (role markers and
/// separators).
const PER_MESSAGE_OVERHEAD: usize = 3;
/// Every reply is primed with `<|start|>assistant<|message|>`.
const REPLY_PRIMING_OVERHEAD: usize = 3;

#[derive(Debug)]
pub struct TokenService;

impl TokenService {
    /// Resolve a model identifier to its tiktoken encoding.
    ///
    /// Accepts either a bare model id or a `provider/model` pair; the provider
    /// prefix is stripped before lookup.
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_agent_runtime::uar::runtime::context::token_service::{TokenEncoding, TokenService};
    /// assert_eq!(TokenService::encoding_for("openai/gpt-4o"), TokenEncoding::O200kBase);
    /// assert_eq!(TokenService::encoding_for("groq/llama-3"), TokenEncoding::Cl100kFallback);
    /// ```
    #[must_use]
    pub fn encoding_for(model: &str) -> TokenEncoding {
        let bare = model.rsplit('/').next().unwrap_or(model);
        let encoding = match bpe_for_model(bare) {
            Ok(bpe) => {
                // `bpe_for_model` returns the singleton for the model's
                // encoding, so pointer identity names it without a second table.
                if std::ptr::eq(bpe, o200k_base_singleton()) {
                    TokenEncoding::O200kBase
                } else if std::ptr::eq(bpe, cl100k_base_singleton()) {
                    TokenEncoding::Cl100kBase
                } else {
                    // A known model on an older encoding (p50k, r50k); count it
                    // with its own tokenizer but report the general case.
                    TokenEncoding::Cl100kBase
                }
            }
            Err(_) => TokenEncoding::Cl100kFallback,
        };
        tracing::debug!(
            name: "context.token.estimate",
            model = model,
            token_encoding = ?encoding,
            token_estimate_fallback = encoding.is_fallback(),
            "Resolved token estimate encoding"
        );
        encoding
    }

    /// Count tokens in `content` with `model`'s encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use universal_agent_runtime::uar::runtime::context::token_service::TokenService;
    /// assert!(TokenService::count("openai/gpt-4o", "hello") > 0);
    /// ```
    #[must_use]
    pub fn count(model: &str, content: &str) -> usize {
        Self::encoding_for(model)
            .bpe()
            .encode_with_special_tokens(content)
            .len()
    }

    /// Count tokens in a message list with `model`'s encoding, including
    /// per-message and reply-priming overhead.
    #[must_use]
    pub fn count_messages(model: &str, messages: &[Message]) -> usize {
        let bpe = Self::encoding_for(model).bpe();
        Self::count_messages_with(bpe, messages)
    }

    fn count_messages_with(bpe: &CoreBPE, messages: &[Message]) -> usize {
        let mut num_tokens = 0;
        for message in messages {
            num_tokens += PER_MESSAGE_OVERHEAD;
            let content_str = message.content.as_text().unwrap_or("");
            num_tokens += bpe.encode_with_special_tokens(content_str).len();
            if let Some(calls) = &message.tool_calls {
                for call in calls {
                    num_tokens += bpe.encode_with_special_tokens(&call.function.name).len();
                    num_tokens += bpe
                        .encode_with_special_tokens(&call.function.arguments)
                        .len();
                }
            }
        }
        num_tokens + REPLY_PRIMING_OVERHEAD
    }

    /// Count tokens for a string when the model is not known to the caller.
    ///
    /// Uses the documented `cl100k_base` fallback. Prefer [`Self::count`]
    /// wherever the model is available.
    #[must_use]
    pub fn estimate_string(content: &str) -> usize {
        let bpe = cl100k_base_singleton();
        bpe.encode_with_special_tokens(content).len()
    }

    /// Count tokens for a message list when the model is not known to the
    /// caller. Prefer [`Self::count_messages`].
    #[must_use]
    pub fn estimate_messages(messages: &[Message]) -> usize {
        let bpe = cl100k_base_singleton();
        let mut num_tokens = 0;

        // Every message follows <|start|>{role/name}\n{content}<|end|>\n
        // Approximate overhead is 3 tokens per message logic + content
        // We'll use a safe approximation.

        for message in messages {
            num_tokens += 3; // overhead
            let content_str = message.content.as_text().unwrap_or("");
            num_tokens += bpe.encode_with_special_tokens(content_str).len();

            // If we had name field, +1 token.
            // If tool calls, we need to count them too.
            if let Some(calls) = &message.tool_calls {
                for call in calls {
                    num_tokens += bpe.encode_with_special_tokens(&call.function.name).len();
                    num_tokens += bpe
                        .encode_with_special_tokens(&call.function.arguments)
                        .len();
                }
            }
        }

        num_tokens += 3; // Every reply is primed with <|start|>assistant<|message|>
        num_tokens
    }
}
