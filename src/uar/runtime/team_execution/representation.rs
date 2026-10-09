use super::TeamExecutionRuntime;
use crate::uar::domain::collaboration::RepresentationGrant;

impl TeamExecutionRuntime {
    pub(crate) async fn invalidate_representation_grant(
        &self,
        owner: &str,
        workspace: &str,
        grant: &RepresentationGrant,
    ) {
        self.manager
            .invalidate_representation_grant(owner, workspace, grant)
            .await;
    }
}
