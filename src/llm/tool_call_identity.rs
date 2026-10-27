//! Private provider identity rules shared by streaming and native admission.

use std::collections::BTreeSet;

const CONFLICT: &str = "Provider tool call identity changed within one stream index";
const MISSING: &str = "Provider native tool call identity is missing";
const DUPLICATE: &str = "Provider native tool call identity is duplicated";

pub(super) fn merge(
    current: &mut Option<String>,
    incoming: Option<&str>,
) -> Result<(), &'static str> {
    let Some(incoming) = incoming else {
        return Ok(());
    };
    if incoming.trim().is_empty() {
        // Preserve unresolved caller-tool fragments for host identity allocation.
        // Native batch admission rejects them before any effects.
        if current.is_none() {
            *current = Some(incoming.to_owned());
        }
        return Ok(());
    }
    match current.as_deref().filter(|id| !id.trim().is_empty()) {
        Some(existing) if existing != incoming => Err(CONFLICT),
        Some(_) => Ok(()),
        None => {
            *current = Some(incoming.to_owned());
            Ok(())
        }
    }
}

pub(super) fn validate_native_batch<'a>(
    calls: impl IntoIterator<Item = (Option<&'a str>, Option<&'a str>)>,
    client_owns: impl Fn(&str) -> bool,
) -> Result<(), &'static str> {
    let mut native_ids = BTreeSet::new();
    for (id, name) in calls {
        if name.is_some_and(&client_owns) {
            continue;
        }
        let id = id.filter(|id| !id.trim().is_empty()).ok_or(MISSING)?;
        if !native_ids.insert(id) {
            return Err(DUPLICATE);
        }
    }
    Ok(())
}

pub(super) fn failure(reason: &'static str) -> anyhow::Error {
    super::ProviderError::new(None, super::ProviderErrorKind::Stream, None, reason).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_authentic_identity_is_preserved_exactly() {
        let mut id = None;
        for fragment in [
            None,
            Some(""),
            Some(" \t"),
            Some(" exact-id "),
            None,
            Some(""),
            Some(" exact-id "),
            Some(" \n"),
        ] {
            merge(&mut id, fragment).unwrap();
        }
        assert_eq!(id.as_deref(), Some(" exact-id "));
        assert_eq!(merge(&mut id, Some("different-private-id")), Err(CONFLICT));
        assert_eq!(id.as_deref(), Some(" exact-id "));
        assert_eq!(failure(CONFLICT).to_string(), CONFLICT);
    }

    #[test]
    fn caller_ids_are_allocated_by_the_host_while_native_ids_must_be_authentic() {
        assert!(
            validate_native_batch(
                [
                    (None, Some("caller_tool")),
                    (Some(""), Some("caller_tool")),
                    (Some("native-id"), Some("native_tool")),
                ],
                |name| name == "caller_tool"
            )
            .is_ok()
        );
        for id in [None, Some(""), Some(" \t")] {
            assert_eq!(
                validate_native_batch([(id, Some("native_tool"))], |_| false),
                Err(MISSING)
            );
        }
        assert_eq!(
            validate_native_batch(
                [
                    (Some("native-id"), Some("first")),
                    (Some("native-id"), Some("second")),
                ],
                |_| false
            ),
            Err(DUPLICATE)
        );
    }
}
