//! Strict parsing and JSON Schema validation for model-supplied tool arguments.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use jsonschema::{ValidationError, Validator};
use serde_json::{Value, json};
use thiserror::Error;

use super::descriptor::ToolAssemblyError;

/// Arguments that could not be parsed or did not satisfy the declared schema.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct InvalidArguments {
    pub message: String,
}

impl InvalidArguments {
    /// Render the stable model-visible failed-tool payload.
    #[must_use]
    pub fn model_result(&self) -> Value {
        json!({
            "type": "invalid_arguments",
            "message": self.message,
        })
    }
}

/// Compile one JSON Schema validator.
///
/// # Errors
///
/// Returns the schema compiler's validation error when `schema` is not a valid
/// JSON Schema for the detected draft.
pub fn compile(schema: &Value) -> Result<Validator, ValidationError<'static>> {
    jsonschema::validator_for(schema)
}

/// Parse raw model arguments and validate the resulting JSON value.
///
/// # Errors
///
/// Returns [`InvalidArguments`] for either malformed JSON or a schema
/// violation. The caller returns this error to the model and must not execute
/// the tool.
pub fn validate(validator: &Validator, arguments_json: &str) -> Result<Value, InvalidArguments> {
    let arguments =
        serde_json::from_str::<Value>(arguments_json).map_err(|error| InvalidArguments {
            message: format!("failed to parse tool arguments: {error}"),
        })?;
    let errors = validator
        .iter_errors(&arguments)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    if errors.is_empty() {
        Ok(arguments)
    } else {
        Err(InvalidArguments {
            message: errors.join("; "),
        })
    }
}

/// Schema compiler with observable call count for assembly verification.
#[derive(Debug, Default)]
pub struct ValidatorCompiler {
    compile_count: AtomicUsize,
}

impl ValidatorCompiler {
    /// Compile a descriptor validator and attach the provider-visible tool name
    /// to any assembly error.
    ///
    /// # Errors
    ///
    /// Returns [`ToolAssemblyError::InvalidSchema`] when the schema cannot be
    /// compiled.
    pub fn compile(
        &self,
        provider_name: &str,
        schema: &Value,
    ) -> Result<Arc<Validator>, ToolAssemblyError> {
        self.compile_count.fetch_add(1, Ordering::Relaxed);
        compile(schema)
            .map(Arc::new)
            .map_err(|error| ToolAssemblyError::InvalidSchema {
                provider_name: provider_name.to_string(),
                message: error.to_string(),
            })
    }

    /// Number of calls made to [`jsonschema::validator_for`].
    #[must_use]
    pub fn compile_count(&self) -> usize {
        self.compile_count.load(Ordering::Relaxed)
    }
}
