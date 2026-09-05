//! Deterministic, budgeted metadata for the complete eligible skill set.

use crate::uar::domain::skills::{Skill, SkillOrigin};
use crate::uar::runtime::context::token_service::TokenService;
use crate::uar::runtime::prompt::{
    Authority, PromptFragment, PromptRole, PromptSection, Retention,
};

const MAX_CATALOG_TOKENS: usize = 10_000;
const UNKNOWN_WINDOW_CHARACTERS: usize = 8_000;
const MAX_DESCRIPTION_CHARACTERS: usize = 1_024;

/// One skill's discoverable metadata; the body is deliberately absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    pub skill_id: String,
    pub title: String,
    pub source: String,
    pub description: String,
    pub suggested: bool,
}

impl From<&Skill> for CatalogEntry {
    fn from(skill: &Skill) -> Self {
        Self {
            skill_id: single_line(&skill.skill_id),
            title: single_line(&skill.title),
            source: if skill.provider_id.is_empty() {
                match skill.origin {
                    SkillOrigin::Builtin => "builtin",
                    SkillOrigin::User => "user",
                }
                .to_string()
            } else {
                single_line(&skill.provider_id)
            },
            description: single_line(&skill.description),
            suggested: false,
        }
    }
}

/// Unit and limit used for a skill catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogBudget {
    Tokens(usize),
    Characters(usize),
}

impl CatalogBudget {
    /// Use two percent of a known window, capped at 10,000 tokens.
    #[must_use]
    pub fn resolve(context_window_tokens: Option<usize>) -> Self {
        match context_window_tokens.filter(|window| *window > 0) {
            Some(window) => Self::Tokens((window / 50).min(MAX_CATALOG_TOKENS)),
            None => Self::Characters(UNKNOWN_WINDOW_CHARACTERS),
        }
    }

    #[must_use]
    pub const fn limit(self) -> usize {
        match self {
            Self::Tokens(limit) | Self::Characters(limit) => limit,
        }
    }

    fn measure(self, model: &str, content: &str) -> usize {
        if content.is_empty() {
            return 0;
        }
        let (start, end) = Authority::Skill.markers();
        let marked = format!("{start}{content}{end}");
        match self {
            Self::Tokens(_) => TokenService::count(model, &marked),
            Self::Characters(_) => marked.chars().count(),
        }
    }
}

/// A catalog that fits its resolved budget, including authority markers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedSkillCatalog {
    pub content: String,
    pub budget: CatalogBudget,
    pub used_units: usize,
    pub included: usize,
    pub omitted: usize,
}

impl RenderedSkillCatalog {
    /// Convert metadata into the fixed-order skill-catalog prompt section.
    #[must_use]
    pub fn into_fragment(self) -> PromptFragment {
        PromptFragment::new(
            "skills.catalog",
            PromptSection::SkillCatalog,
            "eligible_skill_registry",
            Authority::Skill,
            PromptRole::System,
            Retention::Turn,
            self.content,
        )
    }
}

/// Even an omission-count note cannot fit an exceptionally small window.
#[derive(Debug, thiserror::Error)]
#[error("skill catalog budget {available} cannot fit the omission note ({required} units)")]
pub struct CatalogBudgetError {
    pub available: usize,
    pub required: usize,
}

/// Render eligible metadata, trimming every description fairly before omission.
///
/// Whole round-robin passes are found by binary search: each pass removes one
/// character from every non-empty description. This avoids repeatedly tokenizing
/// a thousands-entry catalog once per removed character.
///
/// # Errors
/// Returns an error only when the budget cannot hold the omission-count note.
pub fn render_catalog(
    entries: &[CatalogEntry],
    model: &str,
    context_window_tokens: Option<usize>,
) -> Result<RenderedSkillCatalog, CatalogBudgetError> {
    let budget = CatalogBudget::resolve(context_window_tokens);
    let mut ordered = entries.to_vec();
    ordered.sort_by(|left, right| left.skill_id.cmp(&right.skill_id));
    let descriptions = ordered
        .iter()
        .map(|entry| {
            single_line(&entry.description)
                .chars()
                .take(MAX_DESCRIPTION_CHARACTERS)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let max_rounds = descriptions.iter().map(Vec::len).max().unwrap_or(0);
    let render = |count, rounds| render_lines(&ordered, &descriptions, count, rounds);
    let fits = |content: &str| budget.measure(model, content) <= budget.limit();

    let full = render(ordered.len(), 0);
    let (content, included) = if fits(&full) {
        (full, ordered.len())
    } else {
        let minimum = render(ordered.len(), max_rounds);
        if fits(&minimum) {
            let mut low = 0;
            let mut high = max_rounds;
            let mut best = minimum;
            while low < high {
                let rounds = low + (high - low) / 2;
                let candidate = render(ordered.len(), rounds);
                if fits(&candidate) {
                    best = candidate;
                    high = rounds;
                } else {
                    low = rounds + 1;
                }
            }
            (best, ordered.len())
        } else {
            let compact = render_minimum_lines(&ordered, ordered.len());
            if fits(&compact) {
                return Ok(RenderedSkillCatalog {
                    used_units: budget.measure(model, &compact),
                    content: compact,
                    budget,
                    included: ordered.len(),
                    omitted: 0,
                });
            }

            let note = render_minimum_lines(&ordered, 0);
            if !fits(&note) {
                return Err(CatalogBudgetError {
                    available: budget.limit(),
                    required: budget.measure(model, &note),
                });
            }
            let mut low = 0;
            let mut high = ordered.len();
            let mut best = note;
            let mut included = 0;
            while low < high {
                let count = low + (high - low).div_ceil(2);
                let candidate = render_minimum_lines(&ordered, count);
                if fits(&candidate) {
                    best = candidate;
                    included = count;
                    low = count;
                } else {
                    high = count - 1;
                }
            }
            (best, included)
        }
    };

    Ok(RenderedSkillCatalog {
        used_units: budget.measure(model, &content),
        content,
        budget,
        included,
        omitted: ordered.len().saturating_sub(included),
    })
}

// Source and descriptions are optional in this final compact tier. Titles and
// suggestions remain discoverable; an IDs-only list is not a skill catalog.
fn render_minimum_lines(entries: &[CatalogEntry], count: usize) -> String {
    if entries.is_empty() {
        return String::new();
    }
    let mut lines = vec!["[AVAILABLE SKILLS]".to_string()];
    for entry in entries.iter().take(count) {
        let mut line = single_line(&entry.skill_id);
        let title = single_line(&entry.title);
        if !title.is_empty() {
            line.push(' ');
            line.push_str(&title);
        }
        if entry.suggested {
            line.push_str(" [suggested]");
        }
        lines.push(line);
    }
    if count < entries.len() {
        lines.push(format!("[{} skills omitted]", entries.len() - count));
    }
    lines.join("\n")
}

fn render_lines(
    entries: &[CatalogEntry],
    descriptions: &[Vec<char>],
    count: usize,
    trim_rounds: usize,
) -> String {
    if entries.is_empty() {
        return String::new();
    }
    let mut lines = vec!["[AVAILABLE SKILLS]".to_string()];
    for (entry, description) in entries.iter().zip(descriptions).take(count) {
        let mut line = format!(
            "{} | {} | {}",
            single_line(&entry.skill_id),
            single_line(&entry.title),
            single_line(&entry.source),
        );
        if entry.suggested {
            line.push_str(" [suggested]");
        }
        let remaining = description.len().saturating_sub(trim_rounds);
        if remaining > 0 {
            line.push_str(" — ");
            line.extend(description.iter().take(remaining));
        }
        lines.push(line);
    }
    if count < entries.len() {
        lines.push(format!("[{} skills omitted]", entries.len() - count));
    }
    lines.join("\n")
}

fn single_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
