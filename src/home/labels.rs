use crate::parse::normalize::fold_label;
use crate::types::{HomeGraph, LabelIndex, StoredLabel};

pub fn rebuild_label_index(home: &mut HomeGraph) {
    let mut index = LabelIndex::default();
    for area in &home.areas {
        push_tokens(&mut index, &area.area_id);
        push_tokens(&mut index, &area.name);
        for alias in &area.aliases {
            push_tokens(&mut index, alias);
        }
    }
    for floor in &home.floors {
        push_tokens(&mut index, &floor.floor_id);
        push_tokens(&mut index, &floor.name);
        for alias in &floor.aliases {
            push_tokens(&mut index, alias);
        }
    }
    for (entity_index, entity) in home.entities.iter().enumerate() {
        let mut labels = Vec::new();
        push_label(&mut index, &mut labels, &entity.name, false, entity_index);
        for alias in &entity.aliases {
            push_label(&mut index, &mut labels, alias, false, entity_index);
        }
        for tag in &entity.tags {
            push_label(&mut index, &mut labels, tag, true, entity_index);
        }
        index.entities.insert(entity.entity_id.clone(), labels);
    }
    home.label_index = Some(index);
}

fn push_label(index: &mut LabelIndex, labels: &mut Vec<StoredLabel>, raw: &str, tag: bool, entity_index: usize) {
    let folded = fold_label(raw);
    push_parts(index, &folded, Some(entity_index));
    labels.push(StoredLabel { raw: raw.to_string(), folded, tag });
}

fn push_tokens(index: &mut LabelIndex, raw: &str) {
    push_parts(index, &fold_label(raw), None);
}

fn push_parts(index: &mut LabelIndex, folded: &str, entity_index: Option<usize>) {
    // Split hyphens too — speech tokenizes "Ofen-Timer" as ofen + timer (R-09).
    for part in folded.split([' ', '_', '-']).filter(|part| !part.is_empty()) {
        remember_token(index, part, entity_index);
        // Index the HA-slug form too so "kuche" finds "kueche" (R-09).
        if let Some(collapsed) = collapse_umlaut_digraphs(part) {
            remember_token(index, &collapsed, entity_index);
        }
    }
}

fn remember_token(index: &mut LabelIndex, part: &str, entity_index: Option<usize>) {
    index.tokens.insert(part.to_string());
    let Some(entity_index) = entity_index else {
        return;
    };
    let rows = index.by_token.entry(part.to_string()).or_default();
    if rows.last().copied() != Some(entity_index) {
        rows.push(entity_index);
    }
}

fn collapse_umlaut_digraphs(part: &str) -> Option<String> {
    let chars: Vec<char> = part.chars().collect();
    let mut out = String::with_capacity(chars.len());
    let mut changed = false;
    let mut i = 0;
    while i < chars.len() {
        if matches!(chars[i], 'a' | 'o' | 'u') && chars.get(i + 1) == Some(&'e') {
            out.push(chars[i]);
            i += 2;
            changed = true;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    changed.then_some(out)
}
