# COUNT repair004 source handoff

Packet004 failed strict final-body SHA equality. Pinned repository source vendor/git/liter-llm/crates/liter-llm/src/client/mod.rs build_provider and prepare_request, and src/provider/mod.rs OpenAiCompatibleProvider::strip_model_prefix show the model-hint constructor strips its declared provider prefix. The fixture used new(config, openai/uar-exact-cl100k-json-v1).with_endpoint_profile with wire_model equal to that whole qualified string. The counted profile retained openai/ while the selected adapter removed it. This source-backed mechanism explains a final-body difference; the old receipt did not retain the raw body, so no recovered exact diff is claimed.

Only tests/support/post_lifecycle_count.rs changes: use existing LiterLlmDriver::from_endpoint_profile(config, Some(false), endpoint(base)). That constructor pins the generic adapter using a non-provider hint, matching the declared profile wire model. No new transport, dependency or upstream edit. SHA equality, exact token count, protected tool arguments/results, output cap, one-over refusal, unsupported-count refusal and trusted-builder controls remain. Standalone exact-fit remains UNSUPPORTED. This is a fixture construction correction, not a claim that arbitrary driver/profile pairs have new guarantees.

No tests/checks/builds/reviews ran. Current source remains uncompiled/unexercised.
