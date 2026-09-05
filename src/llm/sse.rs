//! Split OpenAI-style `text/event-stream` payloads.

pub struct SseBuf {
    rest: String,
}

impl Default for SseBuf {
    fn default() -> Self {
        Self { rest: String::new() }
    }
}

impl SseBuf {
    pub fn push(&mut self, chunk: &str) -> Vec<String> {
        self.rest.push_str(chunk);
        self.rest = self.rest.replace("\r\n", "\n");
        let mut events = Vec::new();
        while let Some(idx) = self.rest.find("\n\n") {
            let event: String = self.rest.drain(..=idx + 1).collect();
            if let Some(data) = event_data(&event) {
                events.push(data);
            }
        }
        events
    }
}

pub fn event_data(event: &str) -> Option<String> {
    let mut data = String::new();
    let mut any = false;
    for line in event.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        let Some(rest) = line.strip_prefix("data:") else {
            continue;
        };
        if any {
            data.push('\n');
        }
        data.push_str(rest.strip_prefix(' ').unwrap_or(rest));
        any = true;
    }
    any.then_some(data)
}

pub fn delta_text(data: &str) -> Option<String> {
    if data == "[DONE]" {
        return None;
    }
    let chunk: super::types::UpstreamCompletion = serde_json::from_str(data).ok()?;
    let text = chunk.choices.first().and_then(|choice| choice.delta.as_ref()).map(|delta| delta.text()).unwrap_or_default();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_openai_chunks() {
        let mut buf = SseBuf::default();
        let first = buf.push("data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n");
        assert_eq!(first.len(), 1);
        assert_eq!(delta_text(&first[0]).as_deref(), Some("Hel"));
        let second = buf.push("data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\r\n\r\ndata: [DONE]\r\n\r\n");
        assert_eq!(delta_text(&second[0]).as_deref(), Some("lo"));
        assert_eq!(second[1], "[DONE]");
        assert!(delta_text(&second[1]).is_none());
    }

    #[test]
    fn ignores_comments_and_partial_frames() {
        let mut buf = SseBuf::default();
        assert!(buf.push(": keep-alive\n\ndata: {\"choices\"").is_empty());
        let rest = buf.push(":[{\"delta\":{\"content\":\"x\"}}]}\n\n");
        assert_eq!(delta_text(&rest[0]).as_deref(), Some("x"));
    }
}
