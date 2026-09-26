use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, anyhow, bail};
use serde_json::Value;

use crate::uar::domain::collaboration::{
    CollaborationKind, ImmutableDefinitionRef, PackageManifest,
};

use super::schema::{required_array, required_string, required_u64, validate_reference};

pub(super) fn validate_package_graph(
    manifest: &PackageManifest,
    documents: &BTreeMap<String, Value>,
) -> Result<()> {
    validate_lock_closure(manifest, documents)?;
    validate_package_capabilities(manifest, documents)?;
    validate_definition_graphs(manifest, documents)
}

fn validate_lock_closure(
    manifest: &PackageManifest,
    documents: &BTreeMap<String, Value>,
) -> Result<()> {
    let paths = manifest
        .files
        .iter()
        .map(|file| (file.definition.storage_key(), file.path.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut expected = BTreeSet::new();
    for document in documents.values() {
        let requested_by = required_string(document, "id")?;
        for reference in document_references(document)? {
            let key = reference.storage_key();
            let resolved_path = paths.get(&key).ok_or_else(|| {
                anyhow!(
                    "reference '{}' {} {} is absent from the package inventory",
                    reference.id,
                    reference.version,
                    reference.digest
                )
            })?;
            expected.insert((requested_by.to_owned(), key, (*resolved_path).to_owned()));
        }
    }
    let actual = manifest
        .lock
        .iter()
        .map(|entry| {
            (
                entry.requested_by.clone(),
                entry.reference.storage_key(),
                entry.resolved_path.clone(),
            )
        })
        .collect::<BTreeSet<_>>();
    if expected != actual {
        bail!("package lock must exactly close every immutable definition reference");
    }
    Ok(())
}

fn validate_package_capabilities(
    manifest: &PackageManifest,
    documents: &BTreeMap<String, Value>,
) -> Result<()> {
    let declarations = manifest
        .capability_declarations
        .iter()
        .map(|item| (item.capability.as_str(), item.required))
        .collect::<BTreeMap<_, _>>();
    for capability in &manifest.required_capabilities {
        if declarations.get(capability.as_str()) != Some(&true) {
            bail!("manifest capability '{capability}' is not declared as required");
        }
    }
    for document in documents.values() {
        for capability in required_array(document, "requiredCapabilities")?
            .iter()
            .filter_map(Value::as_str)
        {
            if declarations.get(capability) != Some(&true) {
                bail!("definition capability '{capability}' is absent from the package closure");
            }
        }
    }
    Ok(())
}

fn validate_definition_graphs(
    manifest: &PackageManifest,
    documents: &BTreeMap<String, Value>,
) -> Result<()> {
    let by_key = manifest
        .files
        .iter()
        .map(|file| (file.definition.storage_key(), (&file.kind, file.path.as_str())))
        .collect::<BTreeMap<_, _>>();
    let mut visited = BTreeSet::new();
    let mut visiting = BTreeSet::new();
    for entrypoint in &manifest.entrypoints {
        visit_definition(entrypoint, documents, &by_key, &mut visiting, &mut visited)?;
    }
    for file in &manifest.files {
        let document = documents
            .get(&file.path)
            .ok_or_else(|| anyhow!("definition '{}' is missing", file.path))?;
        match file.kind {
            CollaborationKind::TeamDefinition => validate_team(document, &by_key)?,
            CollaborationKind::WorkflowDefinition => validate_workflow(document)?,
            _ => {}
        }
    }
    Ok(())
}

fn visit_definition(
    reference: &ImmutableDefinitionRef,
    documents: &BTreeMap<String, Value>,
    by_key: &BTreeMap<String, (&CollaborationKind, &str)>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) -> Result<()> {
    let key = reference.storage_key();
    if visited.contains(&key) {
        return Ok(());
    }
    if !visiting.insert(key.clone()) {
        bail!("collaboration definition graph contains a cycle at '{}'", reference.id);
    }
    let (_, path) = by_key
        .get(&key)
        .ok_or_else(|| anyhow!("definition '{}' is absent from the package", reference.id))?;
    let document = documents
        .get(*path)
        .ok_or_else(|| anyhow!("definition source '{}' is absent", path))?;
    for child in document_references(document)? {
        visit_definition(&child, documents, by_key, visiting, visited)?;
    }
    visiting.remove(&key);
    visited.insert(key);
    Ok(())
}

fn validate_team(
    document: &Value,
    by_key: &BTreeMap<String, (&CollaborationKind, &str)>,
) -> Result<()> {
    let members = required_array(document, "members")?;
    let mut roles = BTreeSet::new();
    let mut coordinator_is_agent = false;
    let coordinator = required_string(document, "coordinatorRole")?;
    for member in members {
        let role = required_string(member, "role")?;
        if !roles.insert(role) {
            bail!("team contains duplicate role '{role}'");
        }
        let kind = required_string(member, "kind")?;
        if !matches!(kind, "agent" | "team") {
            bail!("team member '{role}' has unsupported kind '{kind}'");
        }
        let minimum = required_u64(member, "min")?;
        let maximum = required_u64(member, "max")?;
        if minimum > maximum || maximum == 0 || maximum > 16 {
            bail!("team member '{role}' has invalid cardinality");
        }
        let reference = parse_reference(member.get("definition"))?;
        let expected_kind = if kind == "agent" {
            CollaborationKind::AgentDefinition
        } else {
            CollaborationKind::TeamDefinition
        };
        if by_key.get(&reference.storage_key()).map(|item| item.0) != Some(&expected_kind) {
            bail!("team member '{role}' does not resolve to its declared kind");
        }
        coordinator_is_agent |= role == coordinator && kind == "agent";
    }
    if !coordinator_is_agent {
        bail!("coordinatorRole must resolve to an agent member");
    }
    for edge in required_array(document, "communication")? {
        for field in ["fromRole", "toRole"] {
            let role = required_string(edge, field)?;
            if !roles.contains(role) {
                bail!("communication {field} '{role}' is not a declared team role");
            }
        }
        if required_array(edge, "modes")?.is_empty() {
            bail!("communication modes must not be empty");
        }
    }
    let acceptance = document
        .get("taskAcceptance")
        .ok_or_else(|| anyhow!("taskAcceptance is missing"))?;
    for workflow in required_array(acceptance, "allowedWorkflows")? {
        let reference = parse_reference(Some(workflow))?;
        if by_key.get(&reference.storage_key()).map(|item| item.0)
            != Some(&CollaborationKind::WorkflowDefinition)
        {
            bail!("allowed workflow '{}' does not resolve to a workflow definition", reference.id);
        }
    }
    Ok(())
}

fn validate_workflow(document: &Value) -> Result<()> {
    let steps = required_array(document, "steps")?;
    let mut step_ids = BTreeSet::new();
    for step in steps {
        let id = required_string(step, "id")?;
        if !step_ids.insert(id) {
            bail!("workflow contains duplicate step id '{id}'");
        }
    }
    let by_id = steps
        .iter()
        .map(|step| Ok((required_string(step, "id")?, step)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in by_id.keys() {
        visit_workflow(id, &by_id, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn visit_workflow<'a>(
    id: &'a str,
    steps: &BTreeMap<&'a str, &'a Value>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> Result<()> {
    if visited.contains(id) {
        return Ok(());
    }
    if !visiting.insert(id) {
        bail!("workflow dependency graph contains a cycle at '{id}'");
    }
    let step = steps.get(id).ok_or_else(|| anyhow!("workflow step '{id}' is missing"))?;
    for dependency in required_array(step, "dependsOn")? {
        let dependency = dependency
            .as_str()
            .ok_or_else(|| anyhow!("workflow dependencies must be strings"))?;
        if !steps.contains_key(dependency) {
            bail!("workflow step '{id}' depends on unknown step '{dependency}'");
        }
        visit_workflow(dependency, steps, visiting, visited)?;
    }
    visiting.remove(id);
    visited.insert(id);
    Ok(())
}

fn document_references(document: &Value) -> Result<Vec<ImmutableDefinitionRef>> {
    let mut references = Vec::new();
    match required_string(document, "kind")? {
        "AgentDefinition" => collect_reference_array(document, "permittedChildren", &mut references)?,
        "TeamDefinition" => {
            for member in required_array(document, "members")? {
                references.push(parse_reference(member.get("definition"))?);
            }
            let acceptance = document
                .get("taskAcceptance")
                .ok_or_else(|| anyhow!("taskAcceptance is missing"))?;
            collect_reference_array(acceptance, "allowedWorkflows", &mut references)?;
        }
        "WorkflowDefinition" => {}
        kind => bail!("unsupported collaboration kind '{kind}'"),
    }
    Ok(references)
}

fn collect_reference_array(
    document: &Value,
    field: &str,
    output: &mut Vec<ImmutableDefinitionRef>,
) -> Result<()> {
    for value in required_array(document, field)? {
        output.push(parse_reference(Some(value))?);
    }
    Ok(())
}

fn parse_reference(value: Option<&Value>) -> Result<ImmutableDefinitionRef> {
    let reference: ImmutableDefinitionRef = serde_json::from_value(
        value
            .cloned()
            .ok_or_else(|| anyhow!("immutable definition reference is missing"))?,
    )?;
    validate_reference(&reference)?;
    Ok(reference)
}
