//! CH-20: Criterion benchmarks for the per-request hot path — model routing,
//! prompt-dialect detection, and context-strategy trimming. These run on
//! every request (or every turn), so their cost matters at scale even
//! though each call is individually cheap.
//!
//! Run: `cargo bench --bench hot_path`

use std::hint::black_box;
use std::sync::Arc;

use criterion::{Criterion, criterion_group, criterion_main};
use universal_agent_runtime::config::LlmConfig;
use universal_agent_runtime::llm::prompt_dialect::PromptDialect;
use universal_agent_runtime::llm::registry::ProviderRegistry;
use universal_agent_runtime::llm::router::{ModelRouter, RouteRequirements};
use universal_agent_runtime::uar::context::strategy::{apply_strategy, strategy_for_model};

fn bench_prompt_dialect_detect(c: &mut Criterion) {
    let models = [
        "anthropic/claude-opus-4-8",
        "openai/gpt-4o",
        "moonshotai/kimi-k2",
        "zhipuai/glm-4.6",
        "qwen/qwen3-max",
        "minimax/minimax-m2",
        "some-unknown-provider/some-model",
    ];
    c.bench_function("prompt_dialect_detect", |b| {
        b.iter(|| {
            for m in &models {
                black_box(PromptDialect::detect(black_box(m)));
            }
        });
    });
}

fn bench_strategy_for_model(c: &mut Criterion) {
    let windows = [8_000u32, 32_000, 128_000, 200_000, 1_000_000];
    c.bench_function("strategy_for_model", |b| {
        b.iter(|| {
            for w in &windows {
                black_box(strategy_for_model(black_box(*w)));
            }
        });
    });
}

fn bench_apply_strategy_sliding_window(c: &mut Criterion) {
    let strategy = strategy_for_model(128_000);
    // A realistic long-running conversation: 500 turns.
    let messages: Vec<serde_json::Value> = (0..500)
        .map(|i| {
            serde_json::json!({
                "role": if i % 2 == 0 { "user" } else { "assistant" },
                "content": format!("Message number {i} in a long conversation about a complex topic that requires several sentences of realistic-length content to approximate real token counts.")
            })
        })
        .collect();
    c.bench_function("apply_strategy_sliding_window_500_messages", |b| {
        b.iter(|| black_box(apply_strategy(black_box(&messages), black_box(&strategy))));
    });
}

fn bench_router_route(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let router = rt.block_on(async {
        let registry = ProviderRegistry::new();
        for (model, key) in [
            ("openai/gpt-4o", "sk-bench-openai"),
            ("anthropic/claude-3-5-haiku", "sk-bench-anthropic"),
        ] {
            let cfg = LlmConfig {
                model: model.to_string(),
                api_key: Some(key.to_string()),
                ..LlmConfig::default()
            };
            registry.seed_from_llm_config(&cfg).await;
        }
        ModelRouter::new(Arc::new(registry))
    });

    let requirements = RouteRequirements {
        needs_tools: true,
        ..Default::default()
    };

    c.bench_function("model_router_route", |b| {
        b.to_async(&rt)
            .iter(|| async { black_box(router.route(black_box(&requirements)).await) });
    });
}

criterion_group!(
    benches,
    bench_prompt_dialect_detect,
    bench_strategy_for_model,
    bench_apply_strategy_sliding_window,
    bench_router_route,
);
criterion_main!(benches);
