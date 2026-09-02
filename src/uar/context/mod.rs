pub mod strategy;
pub use strategy::{
    ContextStrategy, apply_strategy, keep_first_last, resolve_effective_strategy,
    split_pinned_system, strategy_for_model, trim_count, trim_history,
    trim_history_with_summarization, trim_with_summarization,
};
