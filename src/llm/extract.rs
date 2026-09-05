//! Pull a JSON object out of a model reply (fences or prose).

pub fn json_object(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    let body = fenced(trimmed).unwrap_or(trimmed);
    let start = body.find('{')?;
    let end = body.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(&body[start..=end])
}

fn fenced(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("```")?;
    let rest = rest.strip_prefix("json").or_else(|| rest.strip_prefix("JSON")).unwrap_or(rest);
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    rest.split("```").next().map(str::trim)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_fenced_and_prose() {
        let fenced = "```json\n{\"layer\":\"house\"}\n```";
        assert_eq!(json_object(fenced), Some("{\"layer\":\"house\"}"));
        assert_eq!(json_object("sure {\"ok\":true} thanks"), Some("{\"ok\":true}"));
        assert!(json_object("no object here").is_none());
    }
}
