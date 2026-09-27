use std::sync::Arc;

use universal_agent_runtime::uar::{
    governance::engine::GovernanceEngine,
    runtime::tool_admission::{HostToolAdmissionPort, StandaloneToolAdmissionPort},
};

pub fn explicit_local_authority() -> (
    Arc<GovernanceEngine>,
    Arc<dyn HostToolAdmissionPort>,
) {
    let runtime_epoch = uuid::Uuid::new_v4().to_string();
    (
        Arc::new(
            GovernanceEngine::with_default_permit()
                .expect("explicit embedded integration policy compiles"),
        ),
        Arc::new(StandaloneToolAdmissionPort::new(&runtime_epoch)),
    )
}
