use crate::uar::rag::embeddings::{EmbeddingBackend, EmbeddingConfig, EmbeddingError};
use async_trait::async_trait;
use candle_core::{DType, Device, Error as CandleError, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use std::path::PathBuf;
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::sync::{Mutex, OnceCell};

const CONFIG_NAME: &str = "config.json";
const TOKENIZER_NAME: &str = "tokenizer.json";
const SAFE_WEIGHTS_NAME: &str = "model.safetensors";
const PYTORCH_WEIGHTS_NAME: &str = "pytorch_model.bin";

/// Lazy-loading local embedding backend using a Candle BERT model.
#[derive(Debug)]
pub struct CandleEmbeddingBackend {
    models_dir: PathBuf,
    expected_dimension: usize,
    inner: OnceCell<Arc<Mutex<CandleBackendInner>>>,
}

struct CandleBackendInner {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
    dimensions: usize,
}

impl std::fmt::Debug for CandleBackendInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CandleBackendInner")
            .field("device", &self.device)
            .field("dimensions", &self.dimensions)
            .finish_non_exhaustive()
    }
}

impl CandleEmbeddingBackend {
    pub fn new(config: &EmbeddingConfig) -> Result<Self, EmbeddingError> {
        let models_dir = PathBuf::from(&config.models_dir);
        Ok(Self {
            models_dir,
            expected_dimension: config.vector_dimension,
            inner: OnceCell::new(),
        })
    }

    async fn ensure_loaded(&self) -> Result<&Arc<Mutex<CandleBackendInner>>, EmbeddingError> {
        self.inner
            .get_or_try_init(|| async {
                let device = Self::pick_device()?;
                let inner = tokio::task::spawn_blocking({
                    let models_dir = self.models_dir.clone();
                    let expected_dimension = self.expected_dimension;
                    move || build_inner(models_dir, expected_dimension, device)
                })
                .await
                .map_err(|e| {
                    EmbeddingError::BackendUnavailable(format!(
                        "candle model build task panicked: {e}"
                    ))
                })??;
                Ok(Arc::new(Mutex::new(inner)))
            })
            .await
    }

    fn pick_device() -> Result<Device, EmbeddingError> {
        Ok(Device::Cpu)
    }
}

#[async_trait]
impl EmbeddingBackend for CandleEmbeddingBackend {
    fn backend_name(&self) -> &str {
        "candle"
    }

    fn vector_dimension(&self) -> usize {
        self.expected_dimension
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let inner_mutex = self.ensure_loaded().await?;
        let inner_mutex = Arc::clone(inner_mutex);
        let texts: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        let expected_dimension = self.expected_dimension;

        tokio::task::spawn_blocking(move || {
            let inner = inner_mutex.blocking_lock();
            let mut embeddings = Vec::with_capacity(texts.len());
            for text in &texts {
                let emb = compute_one(&inner, text, expected_dimension)?;
                embeddings.push(emb);
            }
            Ok(embeddings)
        })
        .await
        .map_err(|e| {
            EmbeddingError::BackendUnavailable(format!("candle compute task panicked: {e}"))
        })?
    }
}

fn build_inner(
    models_dir: PathBuf,
    expected_dimension: usize,
    device: Device,
) -> Result<CandleBackendInner, EmbeddingError> {
    if !models_dir.is_dir() {
        return Err(EmbeddingError::BackendUnavailable(format!(
            "models_dir does not exist: {}",
            models_dir.display()
        )));
    }

    let config_path = models_dir.join(CONFIG_NAME);
    let tokenizer_path = models_dir.join(TOKENIZER_NAME);
    let safetensors_path = models_dir.join(SAFE_WEIGHTS_NAME);
    let pytorch_path = models_dir.join(PYTORCH_WEIGHTS_NAME);

    let tokenizer = Tokenizer::from_file(&tokenizer_path).map_err(|e| {
        EmbeddingError::BackendUnavailable(format!(
            "failed to load tokenizer from {}: {e}",
            tokenizer_path.display()
        ))
    })?;

    let config_file = std::fs::File::open(&config_path).map_err(|e| {
        EmbeddingError::BackendUnavailable(format!("failed to open {}: {e}", config_path.display()))
    })?;
    let config: BertConfig = serde_json::from_reader(config_file).map_err(|e| {
        EmbeddingError::BackendUnavailable(format!(
            "failed to parse {}: {e}",
            config_path.display()
        ))
    })?;

    let dimensions = config.hidden_size;
    if dimensions != expected_dimension {
        return Err(EmbeddingError::DimensionMismatch {
            expected: expected_dimension,
            actual: dimensions,
        });
    }

    let weights_path = if safetensors_path.is_file() {
        safetensors_path
    } else if pytorch_path.is_file() {
        pytorch_path
    } else {
        return Err(EmbeddingError::BackendUnavailable(format!(
            "no model weights found in {}",
            models_dir.display()
        )));
    };

    let vb = if weights_path
        .extension()
        .is_some_and(|ext| ext == "safetensors")
    {
        // SAFETY: weights_path points to a file in a user-provided model directory
        // that is not concurrently modified during loading.
        unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path.as_path()], DType::F32, &device)
                .map_err(|e| {
                    EmbeddingError::BackendUnavailable(format!("failed to load weights: {e}"))
                })?
        }
    } else {
        VarBuilder::from_pth(&weights_path, DType::F32, &device).map_err(|e| {
            EmbeddingError::BackendUnavailable(format!("failed to load weights: {e}"))
        })?
    };

    let model = BertModel::load(vb, &config).map_err(|e| {
        EmbeddingError::BackendUnavailable(format!("failed to build BERT model: {e}"))
    })?;

    Ok(CandleBackendInner {
        model,
        tokenizer,
        device,
        dimensions,
    })
}

fn compute_one(
    inner: &CandleBackendInner,
    text: &str,
    expected_dimension: usize,
) -> Result<Vec<f32>, EmbeddingError> {
    let encoding = inner
        .tokenizer
        .encode(text, true)
        .map_err(|e| EmbeddingError::InvalidInput(format!("tokenization failed: {e}")))?;

    let tokens = encoding.get_ids();
    let attention_mask = encoding.get_attention_mask();

    let token_ids = Tensor::new(tokens, &inner.device)
        .map_err(candle_err)?
        .unsqueeze(0)
        .map_err(candle_err)?;
    let attention_mask_tensor = Tensor::new(attention_mask, &inner.device)
        .map_err(candle_err)?
        .unsqueeze(0)
        .map_err(candle_err)?;
    let token_type_ids =
        Tensor::zeros(token_ids.shape(), DType::I64, &inner.device).map_err(candle_err)?;

    let embeddings = inner
        .model
        .forward(&token_ids, &token_type_ids, Some(&attention_mask_tensor))
        .map_err(candle_err)?;

    let pooled = mean_pooling(&embeddings, &attention_mask_tensor).map_err(candle_err)?;
    let normalized = l2_normalize(&pooled).map_err(candle_err)?;

    let vec = normalized
        .squeeze(0)
        .map_err(candle_err)?
        .to_vec1::<f32>()
        .map_err(candle_err)?;

    if vec.len() != expected_dimension {
        return Err(EmbeddingError::DimensionMismatch {
            expected: expected_dimension,
            actual: vec.len(),
        });
    }

    Ok(vec)
}

fn candle_err(e: CandleError) -> EmbeddingError {
    EmbeddingError::RequestFailed(e.to_string())
}

fn mean_pooling(embeddings: &Tensor, attention_mask: &Tensor) -> Result<Tensor, CandleError> {
    let mask = attention_mask
        .to_dtype(DType::F32)?
        .unsqueeze(2)?
        .broadcast_as(embeddings.shape())?;
    let masked = embeddings.mul(&mask)?;
    let summed = masked.sum(1)?;
    let count = mask.sum(1)?.clamp(1e-9, f32::MAX)?;
    summed.div(&count)
}

fn l2_normalize(tensor: &Tensor) -> Result<Tensor, CandleError> {
    let norm = tensor
        .sqr()?
        .sum_keepdim(1)?
        .sqrt()?
        .clamp(1e-12, f32::MAX)?;
    tensor.broadcast_div(&norm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_lazy() {
        let backend = CandleEmbeddingBackend::new(&EmbeddingConfig {
            models_dir: "/tmp/nonexistent-candle-dir".to_string(),
            ..EmbeddingConfig::default()
        })
        .expect("construct lazy backend");
        assert_eq!(backend.backend_name(), "candle");
    }

    #[tokio::test]
    async fn missing_models_dir_fails_on_embed() {
        let backend = CandleEmbeddingBackend::new(&EmbeddingConfig {
            models_dir: std::env::temp_dir()
                .join(format!("candle-missing-{}", uuid::Uuid::new_v4()))
                .to_string_lossy()
                .to_string(),
            ..EmbeddingConfig::default()
        })
        .expect("construct lazy backend");

        let err = backend
            .embed(&["hello"])
            .await
            .expect_err("should fail when models_dir is missing");
        assert!(
            matches!(err, EmbeddingError::BackendUnavailable(_)),
            "expected BackendUnavailable, got {err:?}"
        );
    }
}
