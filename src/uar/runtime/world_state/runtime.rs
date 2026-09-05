//! Trusted host bridge between session history, file access, and pure contributors.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::llm::Message;
use crate::session::Session;
use crate::uar::domain::policy::EffectiveRunPolicy;
use crate::uar::runtime::project_instructions::{ProjectInstructions, ProjectInstructionsConfig};

use super::contributor::{WorldStateBaseline, WorldStateContributor, WorldStateUpdate};
use super::sections::{Clock, WorldStateConfig, WorldStateSnapshot};

/// Per-run host state. Neither contributors nor models receive this write handle.
pub struct WorldStateRuntime {
    session: Session,
    pub instructions: Arc<Mutex<ProjectInstructions>>,
    baseline: Mutex<WorldStateBaseline>,
    cwd: PathBuf,
    workspace_roots: Vec<PathBuf>,
    policy: EffectiveRunPolicy,
    clock: Arc<dyn Clock>,
    config: WorldStateConfig,
}

impl std::fmt::Debug for WorldStateRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldStateRuntime")
            .field("workspace_count", &self.workspace_roots.len())
            .finish_non_exhaustive()
    }
}

impl WorldStateRuntime {
    /// Canonical directory captured when this run's host environment was built.
    pub(crate) fn directory(&self) -> &std::path::Path {
        &self.cwd
    }

    /// Prepare host inputs without extending workspace trust from request data.
    ///
    /// # Errors
    /// Returns invalid configuration, cwd, or admitted instruction read errors.
    pub fn new(
        session: Session,
        cwd: PathBuf,
        instructions_config: ProjectInstructionsConfig,
        config: WorldStateConfig,
        policy: EffectiveRunPolicy,
        clock: Arc<dyn Clock>,
    ) -> std::io::Result<Self> {
        let cwd = cwd.canonicalize()?;
        let mut instructions = ProjectInstructions::discover(instructions_config.clone(), &cwd)?;
        if let Some(previous) = session.project_instructions() {
            instructions.retain_accessed_subtrees(&previous);
        }
        Ok(Self {
            baseline: Mutex::new(session.world_state_baseline()),
            session,
            instructions: Arc::new(Mutex::new(instructions)),
            cwd,
            workspace_roots: instructions_config.trusted_workspaces,
            policy,
            clock,
            config,
        })
    }

    /// Build an owned, side-effect-free contributor from the current host state.
    pub async fn contributor(&self, history_rewritten: bool) -> WorldStateContributor {
        let instructions = self.instructions.lock().await.files();
        WorldStateContributor {
            snapshot: WorldStateSnapshot::capture(
                &self.cwd,
                &self.workspace_roots,
                &self.policy,
                &instructions,
                self.clock.as_ref(),
                self.config,
            ),
            baseline: self.baseline.lock().await.clone(),
            history_rewritten,
        }
    }

    /// Compute a per-call update after reduction and before skill reattachment.
    ///
    /// # Errors
    /// Returns an unrepresentable world-state merge-patch error.
    pub async fn prepare(
        &self,
        history: &[Message],
        rewritten: bool,
    ) -> Result<WorldStateUpdate, super::merge_patch::MergePatchError> {
        let contributor = self.contributor(rewritten).await;
        contributor
            .baseline
            .prepare(&contributor.snapshot, history, rewritten)
    }

    /// Publish only the selected path, never a speculative shadow contribution.
    pub async fn commit(&self, update: &WorldStateUpdate) {
        let instructions = self.instructions.lock().await.clone();
        let mut baseline = self.baseline.lock().await;
        self.session.record_world_state(update, instructions);
        *baseline = update.baseline.clone();
    }
}
