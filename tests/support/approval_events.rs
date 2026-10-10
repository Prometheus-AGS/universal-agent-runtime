//! Exact identities from complete originating approval frames; never select latest.
use serde_json::Value;

pub fn approvals(stream: &str) -> Vec<(String, String)> {
    let normalized = stream.replace("\r\n", "\n");
    let mut seen = std::collections::BTreeSet::new();
    normalized
        .split_inclusive("\n\n")
        .filter_map(|frame| {
            if !frame.ends_with("\n\n")
                || !frame.lines().any(|line| {
                    line.strip_prefix("event:")
                        .is_some_and(|value| value.trim() == "agui.tool_call.approval_required")
                })
            {
                return None;
            }
            let data = frame
                .lines()
                .filter_map(|line| line.strip_prefix("data:").map(str::trim_start))
                .collect::<Vec<_>>()
                .join("\n");
            let body: Value =
                serde_json::from_str(&data).expect("complete approval frame must contain JSON");
            let run = body["request_id"]
                .as_str()
                .expect("approval event must bind a run");
            let id = body["approval_id"]
                .as_str()
                .filter(|id| !id.trim().is_empty())
                .expect("strict cutover requires an originating approval ID");
            let identity = (run.to_owned(), id.to_owned());
            seen.insert(identity.clone()).then_some(identity)
        })
        .collect()
}

pub fn approval_ids(stream: &str, run_id: &str) -> Vec<String> {
    approvals(stream)
        .into_iter()
        .filter_map(|(run, id)| (run == run_id).then_some(id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incomplete_and_replayed_frames_do_not_select_a_new_decision() {
        let frame = "event: agui.tool_call.approval_required\ndata: {\"request_id\":\"run-a\",\"approval_id\":\"exact-a\"}\n\n";
        assert!(approval_ids(frame.trim_end(), "run-a").is_empty());
        assert_eq!(
            approval_ids(&format!("{frame}{frame}"), "run-a"),
            vec!["exact-a"]
        );
        assert!(approval_ids(frame, "run-b").is_empty());
    }
}
