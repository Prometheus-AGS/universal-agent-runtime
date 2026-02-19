use thiserror::Error;

/// Errors produced during UAR-AGENT-MD compilation.
#[derive(Debug, Error)]
pub enum CompileError {
    /// The document is missing YAML frontmatter or it failed to parse.
    #[error("frontmatter error: {0}")]
    Frontmatter(String),

    /// A required section is missing from the document.
    #[error("missing required section: {0}")]
    MissingSection(String),

    /// A section's YAML content failed to deserialize.
    #[error("section '{section}' deserialization error: {message}")]
    SectionDeserialize { section: String, message: String },

    /// The Markdown document has structural issues (e.g., no H1 heading).
    #[error("document structure error: {0}")]
    Structure(String),

    /// A pipeline stage failed validation.
    #[error("stage {stage} ({name}) failed: {message}")]
    StageFailure {
        stage: u8,
        name: String,
        message: String,
    },

    /// Cedar policy compilation error.
    #[error("cedar policy error: {0}")]
    CedarPolicy(String),

    /// Signing / key provider error.
    #[error("signing error: {0}")]
    Signing(String),

    /// Schema validation error (A2UI, A2A JSON Schemas).
    #[error("schema validation error: {0}")]
    SchemaValidation(String),

    /// Storage error (database operations).
    #[error("storage error: {0}")]
    Storage(String),

    /// Generic internal error.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// Convenience alias for compiler results.
pub type CompileResult<T> = Result<T, CompileError>;
