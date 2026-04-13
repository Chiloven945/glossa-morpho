use crate::models::{CandidateItem, EntryDetail, ProjectWorkspace, ValidationIssue};
use regex::Regex;
use std::collections::BTreeSet;
use uuid::Uuid;

pub fn refresh_entry_detail(detail: &mut EntryDetail) {
    detail.issues =
        build_validation_issues(&detail.summary.source_value, &detail.summary.target_value);
    detail.summary.note_count = usize::from(!detail.note.trim().is_empty());
    detail.summary.candidate_count = detail.candidates.len();
}

pub fn sync_summary_from_detail(project: &mut ProjectWorkspace, entry_id: &str) {
    if let Some(detail) = project.details.get(entry_id) {
        if let Some(summary) = project
            .entries
            .iter_mut()
            .find(|entry| entry.id == entry_id)
        {
            *summary = detail.summary.clone();
        }
    }
}

pub fn refresh_project(project: &mut ProjectWorkspace) {
    for detail in project.details.values_mut() {
        refresh_entry_detail(detail);
    }

    for summary in project.entries.iter_mut() {
        if let Some(detail) = project.details.get(&summary.id) {
            *summary = detail.summary.clone();
        }
    }

    project.treemap = crate::state::build_treemap(&project.entries);
    project.stats.total = project.entries.len();
    project.stats.translated = project
        .entries
        .iter()
        .filter(|entry| !entry.target_value.is_empty())
        .count();
    project.stats.missing = project
        .entries
        .iter()
        .filter(|entry| entry.target_value.is_empty())
        .count();
    project.stats.reviewed = project
        .entries
        .iter()
        .filter(|entry| entry.status == "reviewed" || entry.status == "approved")
        .count();
}

pub fn upsert_history_candidate(detail: &mut EntryDetail, value: &str) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }

    if detail
        .candidates
        .iter()
        .any(|candidate| candidate.value.trim() == trimmed)
    {
        return;
    }

    detail.candidates.insert(
        0,
        CandidateItem {
            id: Uuid::new_v4().to_string(),
            source: "history".into(),
            value: value.to_string(),
            score: 0.82,
        },
    );
}

pub fn build_validation_issues(source: &str, target: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if target.is_empty() {
        issues.push(issue(
            "info",
            "Translation is empty and still needs a value.",
        ));
        return issues;
    }

    let source_placeholders = collect_placeholders(source);
    let target_placeholders = collect_placeholders(target);
    if source_placeholders != target_placeholders {
        issues.push(issue(
            "error",
            &format!(
                "Placeholder mismatch. Expected [{}], found [{}].",
                source_placeholders
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
                target_placeholders
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }

    if source.matches('\n').count() != target.matches('\n').count() {
        issues.push(issue(
            "warning",
            "Line break count differs from the upstream string.",
        ));
    }

    if source.chars().next().map(char::is_whitespace)
        != target.chars().next().map(char::is_whitespace)
    {
        issues.push(issue(
            "warning",
            "Leading whitespace differs from the upstream string.",
        ));
    }

    if source.chars().last().map(char::is_whitespace)
        != target.chars().last().map(char::is_whitespace)
    {
        issues.push(issue(
            "warning",
            "Trailing whitespace differs from the upstream string.",
        ));
    }

    if !has_balanced_braces(target) {
        issues.push(issue(
            "error",
            "Unbalanced braces detected in the translation.",
        ));
    }

    issues
}

fn issue(level: &str, message: &str) -> ValidationIssue {
    ValidationIssue {
        id: Uuid::new_v4().to_string(),
        level: level.into(),
        message: message.into(),
    }
}

fn collect_placeholders(value: &str) -> BTreeSet<String> {
    let brace_re = Regex::new(r#"\{[A-Za-z0-9_\.\-]+\}"#).expect("valid brace placeholder regex");
    let mustache_re =
        Regex::new(r#"\{\{\s*[A-Za-z0-9_\.\-]+\s*\}\}"#).expect("valid mustache placeholder regex");
    let printf_re = Regex::new(r#"%(?:\d+\$)?[sdfoxeguc]"#).expect("valid printf regex");

    let mut set = BTreeSet::new();
    for matched in brace_re.find_iter(value) {
        set.insert(matched.as_str().to_string());
    }
    for matched in mustache_re.find_iter(value) {
        set.insert(matched.as_str().replace(' ', ""));
    }
    for matched in printf_re.find_iter(value) {
        set.insert(matched.as_str().to_string());
    }
    set
}

fn has_balanced_braces(value: &str) -> bool {
    let mut depth = 0isize;
    for ch in value.chars() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}
