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
    for entity in &home.entities {
        let mut labels = Vec::new();
        push_label(&mut index, &mut labels, &entity.name, false);
        for alias in &entity.aliases {
            push_label(&mut index, &mut labels, alias, false);
        }
        for tag in &entity.tags {
            push_label(&mut index, &mut labels, tag, true);
        }
        index.entities.insert(entity.entity_id.clone(), labels);
    }
    home.label_index = Some(index);
}

fn push_label(index: &mut LabelIndex, labels: &mut Vec<StoredLabel>, raw: &str, tag: bool) {
    let folded = fold_label(raw);
    push_parts(index, &folded);
    labels.push(StoredLabel { raw: raw.to_string(), folded, tag });
}

fn push_tokens(index: &mut LabelIndex, raw: &str) {
    push_parts(index, &fold_label(raw));
}

fn push_parts(index: &mut LabelIndex, folded: &str) {
    for part in folded.split([' ', '_']).filter(|part| !part.is_empty()) {
        index.tokens.insert(part.to_string());
    }
}
