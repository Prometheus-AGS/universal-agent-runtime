//! Closed provider request plans. No caller-supplied URL, method or authorization header.
use super::{ConnectorAction, ConnectorProvider, bad};
use serde_json::{Value, json};

pub(super) struct RequestPlan {
    pub method: &'static str,
    pub path: String,
    pub body: Option<Value>,
}

fn segment(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value.len() <= 120
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

pub(super) fn valid_target(provider: ConnectorProvider, target: &str) -> bool {
    match provider {
        ConnectorProvider::Github => {
            let parts: Vec<_> = target.split('/').collect();
            parts.len() == 2 && parts.iter().all(|part| segment(part))
        }
        ConnectorProvider::Slack | ConnectorProvider::Jira => segment(target),
        ConnectorProvider::Notion => {
            target.len() == 36
                && target.chars().enumerate().all(|(index, ch)| {
                    if matches!(index, 8 | 13 | 18 | 23) {
                        ch == '-'
                    } else {
                        ch.is_ascii_hexdigit()
                    }
                })
        }
    }
}

fn string<'a>(payload: &'a Value, name: &str) -> Result<&'a str, super::CollaborationError> {
    payload
        .get(name)
        .and_then(Value::as_str)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| bad("CONNECTOR_PAYLOAD_INVALID"))
}

pub(super) fn plan(
    provider: ConnectorProvider,
    action: ConnectorAction,
    target: &str,
    payload: &Value,
) -> Result<RequestPlan, super::CollaborationError> {
    if !valid_target(provider, target) {
        return Err(bad("CONNECTOR_TARGET_INVALID"));
    }
    use ConnectorAction as A;
    use ConnectorProvider as P;
    match (provider, action) {
        (_, A::Draft) => Ok(RequestPlan {
            method: "LOCAL",
            path: String::new(),
            body: Some(payload.clone()),
        }),
        (P::Github, A::Read) => {
            let number = payload
                .get("issueNumber")
                .and_then(Value::as_u64)
                .filter(|n| *n > 0)
                .ok_or_else(|| bad("CONNECTOR_ISSUE_NUMBER_INVALID"))?;
            Ok(RequestPlan {
                method: "GET",
                path: format!("/repos/{target}/issues/{number}"),
                body: None,
            })
        }
        (P::Github, A::Publish) => {
            let title = string(payload, "title")?;
            let body = string(payload, "body")?;
            Ok(RequestPlan {
                method: "POST",
                path: format!("/repos/{target}/issues"),
                body: Some(json!({"title":title,"body":body})),
            })
        }
        (P::Slack, A::Read) => Ok(RequestPlan {
            method: "GET",
            path: format!("/api/conversations.history?channel={target}"),
            body: None,
        }),
        (P::Slack, A::Send) => Ok(RequestPlan {
            method: "POST",
            path: "/api/chat.postMessage".into(),
            body: Some(json!({"channel":target,"text":string(payload,"text")?})),
        }),
        (P::Notion, A::Read) => Ok(RequestPlan {
            method: "GET",
            path: format!("/v1/pages/{target}"),
            body: None,
        }),
        (P::Notion, A::Write) => {
            let properties = payload
                .get("properties")
                .filter(|v| v.is_object())
                .ok_or_else(|| bad("CONNECTOR_PAYLOAD_INVALID"))?;
            Ok(RequestPlan {
                method: "PATCH",
                path: format!("/v1/pages/{target}"),
                body: Some(json!({"properties":properties})),
            })
        }
        (P::Notion, A::Publish) => {
            let properties = payload
                .get("properties")
                .filter(|v| v.is_object())
                .ok_or_else(|| bad("CONNECTOR_PAYLOAD_INVALID"))?;
            Ok(RequestPlan {
                method: "POST",
                path: "/v1/pages".into(),
                body: Some(
                    json!({"parent":{"type":"page_id","page_id":target},"properties":properties}),
                ),
            })
        }
        (P::Jira, A::Read) => {
            let key = string(payload, "issueKey")?;
            if !key.starts_with(&format!("{target}-")) || !segment(key) {
                return Err(bad("CONNECTOR_TARGET_MISMATCH"));
            }
            Ok(RequestPlan {
                method: "GET",
                path: format!("/rest/api/3/issue/{key}"),
                body: None,
            })
        }
        (P::Jira, A::Write) => {
            let title = string(payload, "title")?;
            let body = string(payload, "body")?;
            Ok(RequestPlan {
                method: "POST",
                path: "/rest/api/3/issue".into(),
                body: Some(json!({"fields":{"project":{"key":target},"summary":title,
                    "issuetype":{"name":"Task"},"description":{"type":"doc","version":1,
                    "content":[{"type":"paragraph","content":[{"type":"text","text":body}]}]}}})),
            })
        }
        _ => Err(bad("CONNECTOR_ACTION_UNSUPPORTED")),
    }
}
