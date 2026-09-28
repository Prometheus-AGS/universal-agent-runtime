//! Owner-scoped durable team inbox over the existing catalog CAS.

use chrono::Utc;

use crate::uar::domain::{
    collaboration::CollaborationCatalogState,
    team_mailbox::{
        EnqueueTeamMessageRequest, TeamInboxMessage, TeamMailboxCommandReceipt, TeamMessageStatus,
    },
    team_planning::TeamInstance,
};

use super::{
    service::{CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, validate_owner},
    validation::{request_digest, validate_id},
};

impl CollaborationCatalogService {
    pub async fn enqueue_team_message(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
        request: EnqueueTeamMessageRequest,
    ) -> Result<TeamInboxMessage, CollaborationError> {
        validate_scope(owner_id, workspace_id, team_id)?;
        validate_id(&request.command_id)?;
        validate_id(&request.message_id)?;
        validate_id(&request.recipient_member_id)?;
        if let Some(task_id) = &request.task_id {
            validate_id(task_id)?;
        }
        let digest = request_digest(&request)?;
        let receipt_key = command_key(owner_id, workspace_id, &request.command_id);
        let message_key = message_key(owner_id, workspace_id, team_id, &request.message_id);

        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(receipt) = current.team_mailbox_command_receipts.get(&receipt_key) {
                if receipt.operation != "enqueue-message"
                    || receipt.request_digest != digest
                    || receipt.team_id != team_id
                {
                    return Err(CollaborationError::Conflict(
                        "commandId was already used for a different mailbox operation or payload"
                            .to_owned(),
                    ));
                }
                let message = current
                    .team_inbox_messages
                    .get(&message_key_for_receipt(receipt))
                    .cloned()
                    .ok_or_else(|| {
                        CollaborationError::Storage(
                            "mailbox command receipt has no durable message".to_owned(),
                        )
                    })?;
                let team = scoped_team(&current, owner_id, workspace_id, team_id)?;
                TeamMailboxGrant::for_delivery(team, &message)?;
                return Ok(message);
            }
            if current.team_inbox_messages.contains_key(&message_key) {
                return Err(CollaborationError::Conflict(
                    "messageId already exists for this team".to_owned(),
                ));
            }
            let team = scoped_team(&current, owner_id, workspace_id, team_id)?;
            let grant = TeamMailboxGrant::for_operator(&team, &request)?;
            let now = Utc::now();
            let message = TeamInboxMessage {
                message_id: request.message_id.clone(),
                owner_id: owner_id.to_owned(),
                workspace_id: workspace_id.to_owned(),
                team_id: team_id.to_owned(),
                sender_owner_id: owner_id.to_owned(),
                recipient_member_id: request.recipient_member_id.clone(),
                recipient_member_revision: grant.recipient_member_revision,
                task_id: request.task_id.clone(),
                task_epoch: grant.task_epoch,
                mode: request.mode,
                content: request.content.clone(),
                status: TeamMessageStatus::Accepted,
                accepted_at: now,
                delivered_at: None,
                processed_at: None,
                processed_turn_id: None,
            };
            let mut next = current.clone();
            next.generation = current.generation.saturating_add(1);
            next.team_inbox_messages
                .insert(message_key.clone(), message.clone());
            next.team_mailbox_command_receipts.insert(
                receipt_key.clone(),
                TeamMailboxCommandReceipt {
                    owner_id: owner_id.to_owned(),
                    workspace_id: workspace_id.to_owned(),
                    command_id: request.command_id.clone(),
                    request_digest: digest.clone(),
                    operation: "enqueue-message".to_owned(),
                    team_id: team_id.to_owned(),
                    message_id: request.message_id.clone(),
                    committed_at: now,
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(message);
            }
        }
        Err(CollaborationError::Conflict(
            "team mailbox changed during enqueue".to_owned(),
        ))
    }

    pub async fn list_team_messages(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
    ) -> Result<Vec<TeamInboxMessage>, CollaborationError> {
        validate_scope(owner_id, workspace_id, team_id)?;
        let state = self.load_state().await?;
        scoped_team(&state, owner_id, workspace_id, team_id)?;
        let mut messages: Vec<_> = state
            .team_inbox_messages
            .into_values()
            .filter(|message| {
                message.owner_id == owner_id
                    && message.workspace_id == workspace_id
                    && message.team_id == team_id
            })
            .collect();
        messages.sort_by(|left, right| {
            left.accepted_at
                .cmp(&right.accepted_at)
                .then_with(|| left.message_id.cmp(&right.message_id))
        });
        Ok(messages)
    }

    pub async fn get_team_message(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
        message_id: &str,
    ) -> Result<TeamInboxMessage, CollaborationError> {
        validate_scope(owner_id, workspace_id, team_id)?;
        validate_id(message_id)?;
        let state = self.load_state().await?;
        scoped_team(&state, owner_id, workspace_id, team_id)?;
        state
            .team_inbox_messages
            .get(&message_key(owner_id, workspace_id, team_id, message_id))
            .cloned()
            .ok_or_else(|| CollaborationError::NotFound(message_id.to_owned()))
    }

    /// Called only after the trusted host has made the message available to its recipient.
    /// The host must not call this for an owner inspecting the inbox.
    #[allow(dead_code)]
    pub(crate) async fn mark_team_message_delivered(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
        message_id: &str,
        recipient_member_id: &str,
    ) -> Result<TeamInboxMessage, CollaborationError> {
        self.advance_message(
            owner_id,
            workspace_id,
            team_id,
            message_id,
            recipient_member_id,
            None,
        )
        .await
    }

    /// Called only after a trusted recipient turn consumes the delivered input.
    #[allow(dead_code)]
    pub(crate) async fn mark_team_message_processed(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
        message_id: &str,
        recipient_member_id: &str,
        turn_id: &str,
    ) -> Result<TeamInboxMessage, CollaborationError> {
        validate_id(turn_id)?;
        self.advance_message(
            owner_id,
            workspace_id,
            team_id,
            message_id,
            recipient_member_id,
            Some(turn_id),
        )
        .await
    }

    async fn advance_message(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
        message_id: &str,
        recipient_member_id: &str,
        turn_id: Option<&str>,
    ) -> Result<TeamInboxMessage, CollaborationError> {
        validate_scope(owner_id, workspace_id, team_id)?;
        validate_id(message_id)?;
        validate_id(recipient_member_id)?;
        let key = message_key(owner_id, workspace_id, team_id, message_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            let team = scoped_team(&current, owner_id, workspace_id, team_id)?;
            let mut message = current
                .team_inbox_messages
                .get(&key)
                .cloned()
                .ok_or_else(|| CollaborationError::NotFound(message_id.to_owned()))?;
            if message.recipient_member_id != recipient_member_id {
                return Err(CollaborationError::Conflict(
                    "recipient does not match the durable message".to_owned(),
                ));
            }
            TeamMailboxGrant::for_delivery(&team, &message)?;
            match (turn_id, message.status) {
                (None, TeamMessageStatus::Accepted) => {
                    message.status = TeamMessageStatus::Delivered;
                    message.delivered_at = Some(Utc::now());
                }
                (None, _) => return Ok(message),
                (Some(id), TeamMessageStatus::Delivered) => {
                    message.status = TeamMessageStatus::Processed;
                    message.processed_at = Some(Utc::now());
                    message.processed_turn_id = Some(id.to_owned());
                }
                (Some(id), TeamMessageStatus::Processed)
                    if message.processed_turn_id.as_deref() == Some(id) =>
                {
                    return Ok(message);
                }
                (Some(_), _) => {
                    return Err(CollaborationError::Conflict(
                        "message must be delivered before a turn can process it".to_owned(),
                    ));
                }
            }
            let mut next = current.clone();
            next.generation = current.generation.saturating_add(1);
            next.team_inbox_messages
                .insert(key.clone(), message.clone());
            if self.cas(current.generation, &next).await? {
                return Ok(message);
            }
        }
        Err(CollaborationError::Conflict(
            "team mailbox changed during receipt update".to_owned(),
        ))
    }
}

/// A narrow disclosure grant derived from current team state, never from message text.
struct TeamMailboxGrant {
    recipient_member_revision: u64,
    task_epoch: Option<u64>,
}

impl TeamMailboxGrant {
    fn for_operator(
        team: &TeamInstance,
        request: &EnqueueTeamMessageRequest,
    ) -> Result<Self, CollaborationError> {
        let member = team
            .members
            .iter()
            .find(|member| member.id == request.recipient_member_id && member.status != "stopped")
            .ok_or_else(|| CollaborationError::NotFound(request.recipient_member_id.clone()))?;
        let task_epoch = match (&request.task_id, request.expected_task_epoch) {
            (None, None) => None,
            (Some(task_id), Some(epoch)) if epoch > 0 => {
                let task = team
                    .tasks
                    .iter()
                    .find(|task| task.id == *task_id)
                    .ok_or_else(|| CollaborationError::NotFound(task_id.clone()))?;
                if task.status != "ready"
                    || task.assignee_member_id.as_deref() != Some(member.id.as_str())
                    || task.ownership_epoch != epoch
                {
                    return Err(CollaborationError::Conflict(
                        "task assignment or ownership epoch changed".to_owned(),
                    ));
                }
                let authority = task.assignment_authority.as_ref().ok_or_else(|| {
                    CollaborationError::Conflict(
                        "task has no current assignment authority".to_owned(),
                    )
                })?;
                if authority.member_id != member.id
                    || authority.workspace_id != team.workspace_id
                    || authority.role != task.role
                    || authority.role != member.role
                    || authority.binding_id != team.binding.id
                    || authority.binding_revision != team.binding.revision
                    || authority.ownership_epoch != epoch
                    || authority.can_execute
                    || authority.can_use_tools
                {
                    return Err(CollaborationError::Conflict(
                        "task assignment authority no longer matches its narrowed scope".to_owned(),
                    ));
                }
                Some(epoch)
            }
            _ => {
                return Err(CollaborationError::Invalid(
                    "taskId and positive expectedTaskEpoch must be supplied together".to_owned(),
                ));
            }
        };
        Ok(Self {
            recipient_member_revision: member.revision,
            task_epoch,
        })
    }

    fn for_delivery(
        team: &TeamInstance,
        message: &TeamInboxMessage,
    ) -> Result<Self, CollaborationError> {
        if message.owner_id != team.owner_id
            || message.workspace_id != team.workspace_id
            || message.team_id != team.id
            || message.sender_owner_id != team.owner_id
        {
            return Err(CollaborationError::Conflict(
                "message scope no longer matches the team".to_owned(),
            ));
        }
        let request = EnqueueTeamMessageRequest {
            command_id: String::new(),
            message_id: message.message_id.clone(),
            recipient_member_id: message.recipient_member_id.clone(),
            task_id: message.task_id.clone(),
            expected_task_epoch: message.task_epoch,
            mode: message.mode,
            content: serde_json::Value::Null,
        };
        let grant = Self::for_operator(team, &request)?;
        if grant.recipient_member_revision != message.recipient_member_revision {
            return Err(CollaborationError::Conflict(
                "recipient membership revision changed".to_owned(),
            ));
        }
        Ok(grant)
    }
}

fn validate_scope(
    owner_id: &str,
    workspace_id: &str,
    team_id: &str,
) -> Result<(), CollaborationError> {
    validate_owner(owner_id)?;
    validate_id(workspace_id)?;
    validate_id(team_id)?;
    Ok(())
}

fn scoped_team<'a>(
    state: &'a CollaborationCatalogState,
    owner_id: &str,
    workspace_id: &str,
    team_id: &str,
) -> Result<&'a TeamInstance, CollaborationError> {
    state
        .team_instances
        .get(&team_key(owner_id, workspace_id, team_id))
        .ok_or_else(|| CollaborationError::NotFound(team_id.to_owned()))
}

fn team_key(owner_id: &str, workspace_id: &str, team_id: &str) -> String {
    format!("{owner_id}\u{1f}{workspace_id}\u{1f}{team_id}")
}

fn command_key(owner_id: &str, workspace_id: &str, command_id: &str) -> String {
    format!("{owner_id}\u{1f}{workspace_id}\u{1f}{command_id}")
}

fn message_key(owner_id: &str, workspace_id: &str, team_id: &str, message_id: &str) -> String {
    format!("{owner_id}\u{1f}{workspace_id}\u{1f}{team_id}\u{1f}{message_id}")
}

fn message_key_for_receipt(receipt: &TeamMailboxCommandReceipt) -> String {
    message_key(
        &receipt.owner_id,
        &receipt.workspace_id,
        &receipt.team_id,
        &receipt.message_id,
    )
}
