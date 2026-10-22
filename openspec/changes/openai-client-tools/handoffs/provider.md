# Provider controls handoff

Phase: Execute, active waypoint function-first-release-certification. Verification lock CLOSED. Commands were source inspection and implementation writing only; no tests, checks, formatter, compiler, build, or review gate ran. No dependency updates.

Owned source changes:
- src/llm/mod.rs: ToolChoice { Auto, None, Required, Named(String) }, optional LlmRequest.tool_choice and parallel_tool_calls. validate_tool_controls refuses required without declarations and named undeclared tools.
- src/llm/liter_driver.rs: caller parallel override and typed Liter tool-choice mapping; existing unspecified behavior retained. Explicit controls fail with provider capability errors where pinned adapters cannot be relied upon to preserve semantics. Exact endpoint profiles use their existing final-wire field validation. Strict fields retained by pinned typed FunctionDefinition for OpenAI-compatible requests. Non-OpenAI adapters without an exact endpoint profile refuse strict rather than discard it; Anthropic adapter schema rewrites are refused when root type is not object. extra_params cannot override typed tool controls.
- src/llm/anthropic_driver.rs: native tool choice mapping (required => any), parallel => inverse disable_parallel_tool_use, strict preservation. Malformed typed declarations fail before conversion. Forced choice with enabled thinking fails explicitly. parallel plus none fails explicitly because native none has no meaningful parallel control.
- src/llm/anthropic_types.rs: native wire choice variants carry disable_parallel_tool_use; none added; ToolDef strict optional added.

Shared fields are public Option<ToolChoice> and Option<bool>. ToolChoice serde uses tagged type/content name with snake_case variants. Native ToolChoice Auto/Any now have named optional fields. Existing ToolDef initializer in anthropic_cache.rs needs strict: None; lead notified. All other LlmRequest literals outside ownership require both optional fields (lead has full list).

Source state is shared uncommitted checkout; no commit or push. Lead must capture whole-source identity at the authorized inventory boundary. No review or executable acceptance evidence exists. Risks: provider/model actual control support remains upstream-enforced; unsupported Liter adapters fail explicitly for caller controls rather than silently falling back to auto. This can reject a capable adapter until its exact semantics are supported or an endpoint profile is supplied. Native Anthropic driver continues its existing budget-contract refusal.
