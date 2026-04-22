// ================================================================
// Skill Mutation Policies — Self-Learning Pipeline Governance
// ================================================================
//
// Gates all write operations against skill artifacts from the
// prometheus-skill-system self-learning pipeline:
// - prompt optimization (dspy-rs writes back to SKILL.md)
// - skill generation (PMPO creates new skills from gap detection)
// - skill promotion (generated skills move from staging to active)
// - trace capture (execution data collection)
//
// Environment is passed in context. Default: "development".
// Set PROMETHEUS_ENVIRONMENT=staging|production for stricter enforcement.
// ================================================================

// --- DEVELOPMENT: permit all skill operations ---

permit(
    principal,
    action == Action::"skill.mutate",
    resource
) when {
    context.environment == "development"
};

permit(
    principal,
    action == Action::"skill.generate",
    resource
) when {
    context.environment == "development"
};

permit(
    principal,
    action == Action::"skill.promote",
    resource
) when {
    context.environment == "development"
};

// --- STAGING: require validation for mutations ---

permit(
    principal,
    action == Action::"skill.mutate",
    resource
) when {
    context.environment == "staging" &&
    context has validation_passed &&
    context.validation_passed == true
};

// --- STAGING: require human approval + test pass rate for promotions ---

permit(
    principal,
    action == Action::"skill.promote",
    resource
) when {
    context.environment == "staging" &&
    context has human_approved &&
    context.human_approved == true &&
    context has test_pass_rate &&
    context.test_pass_rate >= 95
};

// --- TRACE CAPTURE: always allow with governance consent ---

permit(
    principal,
    action == Action::"trace.capture",
    resource
) when {
    context has governance_consent &&
    context.governance_consent == true
};

// --- PRODUCTION: deny all skill mutations ---

forbid(
    principal,
    action == Action::"skill.mutate",
    resource
) when {
    context.environment == "production"
};

forbid(
    principal,
    action == Action::"skill.generate",
    resource
) when {
    context.environment == "production"
};
