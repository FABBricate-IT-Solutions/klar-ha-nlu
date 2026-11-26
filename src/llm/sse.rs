//! Split OpenAI-style `text/event-stream` payloads on byte boundaries.

use super::types::LlmError;

/// Cap on buffered + drained SSE bytes for one upstream response (acceptance: 16 MiB).
pub const MAX_STREAM_BYTES: usize = 16 * 1024 * 1024;

#[derive(Default)]
pub struct SseBuf {
    rest: Vec<u8>,
    /// Total bytes accepted from the upstream body for this stream.
    seen: usize,
}

impl SseBuf {
    pub fn push(&mut self, chunk: &[u8]) -> Result<Vec<String>, LlmError> {
        self.account(chunk.len())?;
        self.rest.extend_from_slice(chunk);
        self.drain_events(false)
    }

    /// Emit a trailing frame that never got a blank-line terminator.
    pub fn finish(&mut self) -> Result<Vec<String>, LlmError> {
        self.drain_events(true)
    }

    fn account(&mut self, added: usize) -> Result<(), LlmError> {
        let next = self.seen.saturating_add(added);
        if next > MAX_STREAM_BYTES || self.rest.len().saturating_add(added) > MAX_STREAM_BYTES {
            return Err(LlmError::TooLarge);
        }
        self.seen = next;
        Ok(())
    }

    fn drain_events(&mut self, flush: bool) -> Result<Vec<String>, LlmError> {
        let mut events = Vec::new();
        loop {
            match find_event_boundary(&self.rest) {
                Some((end, consume)) => {
                    let raw: Vec<u8> = self.rest.drain(..consume).collect();
                    let event = &raw[..end];
                    if let Some(data) = event_data_bytes(event)? {
                        events.push(data);
                    }
                }
                None if flush && !self.rest.is_empty() => {
                    let raw = std::mem::take(&mut self.rest);
                    if let Some(data) = event_data_bytes(&raw)? {
                        events.push(data);
                    }
                    break;
                }
                None => break,
            }
        }
        Ok(events)
    }
}

/// Blank line ends an event. Accepts `\n\n`, `\r\n\r\n`, `\r\n\n`, and `\n\r\n`.
fn find_event_boundary(buf: &[u8]) -> Option<(usize, usize)> {
    let mut i = 0;
    while i < buf.len() {
        if buf[i] == b'\n' {
            if i + 1 < buf.len() && buf[i + 1] == b'\n' {
                return Some((i, i + 2));
            }
            if i + 2 < buf.len() && buf[i + 1] == b'\r' && buf[i + 2] == b'\n' {
                return Some((i, i + 3));
            }
        } else if buf[i] == b'\r' && i + 1 < buf.len() && buf[i + 1] == b'\n' {
            if i + 2 < buf.len() && buf[i + 2] == b'\n' {
                return Some((i, i + 3));
            }
            if i + 3 < buf.len() && buf[i + 2] == b'\r' && buf[i + 3] == b'\n' {
                return Some((i, i + 4));
            }
        }
        i += 1;
    }
    None
}

fn event_data_bytes(event: &[u8]) -> Result<Option<String>, LlmError> {
    let text = std::str::from_utf8(event).map_err(|_| LlmError::Response)?;
    Ok(event_data(text))
}

pub fn event_data(event: &str) -> Option<String> {
    let mut data = String::new();
    let mut any = false;
    for line in event.lines() {
        let line = line.trim_end_matches('\r').trim_end();
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

pub fn delta_tool_calls(data: &str) -> Option<Vec<super::types::UpstreamToolCall>> {
    if data == "[DONE]" {
        return None;
    }
    let chunk: super::types::UpstreamCompletion = serde_json::from_str(data).ok()?;
    let calls = chunk.choices.first().and_then(|choice| choice.delta.as_ref()).and_then(|delta| delta.tool_calls.clone())?;
    if calls.is_empty() {
        None
    } else {
        Some(calls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_openai_chunks() {
        let mut buf = SseBuf::default();
        let first = buf.push(b"data: {\"choices\":[{\"delta\":{\"content\":\"Hel\"}}]}\n\n").unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(delta_text(&first[0]).as_deref(), Some("Hel"));
        let second = buf.push(b"data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\r\n\r\ndata: [DONE]\r\n\r\n").unwrap();
        assert_eq!(delta_text(&second[0]).as_deref(), Some("lo"));
        assert_eq!(second[1], "[DONE]");
        assert!(delta_text(&second[1]).is_none());
        let tools = buf
            .push(
                b"data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"c1\",\"function\":{\"name\":\"get_entity\",\"arguments\":\"{}\"}}]}}]}\n\n",
            )
            .unwrap();
        let calls = delta_tool_calls(&tools[0]).expect("tool delta");
        assert_eq!(calls[0].id.as_deref(), Some("c1"));
        assert_eq!(calls[0].function.as_ref().and_then(|fn_| fn_.name.as_deref()), Some("get_entity"));
        assert!(delta_text(&tools[0]).is_none());
    }

    #[test]
    fn ignores_comments_and_partial_frames() {
        let mut buf = SseBuf::default();
        assert!(buf.push(b": keep-alive\n\ndata: {\"choices\"").unwrap().is_empty());
        let rest = buf.push(b":[{\"delta\":{\"content\":\"x\"}}]}\n\n").unwrap();
        assert_eq!(delta_text(&rest[0]).as_deref(), Some("x"));
    }

    #[test]
    fn split_umlaut_stays_intact() {
        // "ü" is c3 bc — each half alone is invalid UTF-8; lossy decode would yield U+FFFD.
        let mut buf = SseBuf::default();
        let prefix = b"data: {\"choices\":[{\"delta\":{\"content\":\"";
        let suffix = b"\"}}]}\n\n";
        assert!(buf.push(prefix).unwrap().is_empty());
        assert!(buf.push(&[0xc3]).unwrap().is_empty());
        let events = buf.push(&[0xbc]).unwrap();
        assert!(events.is_empty(), "no frame yet without terminator");
        let done = buf.push(suffix).unwrap();
        assert_eq!(delta_text(&done[0]).as_deref(), Some("ü"));
        assert!(!delta_text(&done[0]).unwrap().contains('\u{FFFD}'));
    }

    #[test]
    fn finish_emits_trailing_event_without_blank_line() {
        let mut buf = SseBuf::default();
        assert!(buf.push(b"data: {\"choices\":[{\"delta\":{\"content\":\"end\"}}]}").unwrap().is_empty());
        let flushed = buf.finish().unwrap();
        assert_eq!(delta_text(&flushed[0]).as_deref(), Some("end"));
    }

    #[test]
    fn rejects_stream_over_16_mib() {
        let mut buf = SseBuf::default();
        let chunk = vec![b'a'; 1024 * 1024];
        for _ in 0..16 {
            assert!(buf.push(&chunk).unwrap().is_empty());
        }
        let err = buf.push(b"x").unwrap_err();
        assert!(matches!(err, LlmError::TooLarge));
    }
}
