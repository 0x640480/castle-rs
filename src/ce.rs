//! Typed encoder/decoder for the `ce` event-stream blob — a compact binary
//! record of DOM interactions on the page.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// One recorded DOM-interaction event.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Event {
    /// Event class. Classes with a registry handler carry extra payload
    /// bytes; see [`gb_handler_bytes`].
    pub class: u8,

    /// DOM tag index into [`NODE_TAGS`]. `None` means no associated node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_idx: Option<u8>,

    /// Input-type index into [`INPUT_TYPES`]. Only meaningful with `node_idx`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_idx: Option<u8>,

    /// Low byte of the event timestamp. `None` means absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u8>,

    /// Registry-handler output (length per [`gb_handler_bytes`]); empty otherwise.
    /// Serialized as a JSON number array; omitted when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registry_bytes: Vec<u8>,
}

/// Number of registry bytes a class's handler appends, if any.
pub fn gb_handler_bytes(class: u8) -> Option<usize> {
    match class {
        3 => Some(1),  // clipboard flag bits
        19 => Some(2), // key-event flags
        22 => Some(1), // delay marker
        23 => Some(1), // event flags
        _ => None,
    }
}

/// DOM-tag lookup table — index by [`Event::node_idx`].
pub const NODE_TAGS: &[&str] = &[
    "div", "span", "p", "a", "img", "button", "input", "form", "label", "select", "textarea", "ul",
    "ol", "li", "h1", "h2", "h3", "table", "tr", "td", "th", "header", "footer", "nav", "main",
    "section", "article", "aside", "canvas", "video", "audio", "iframe", "script", "style",
];

/// `HTMLInputElement.type` lookup table — index by [`Event::input_idx`].
pub const INPUT_TYPES: &[&str] = &[
    "text",
    "password",
    "email",
    "number",
    "tel",
    "url",
    "search",
    "date",
    "time",
    "datetime-local",
    "month",
    "week",
    "color",
    "file",
    "range",
    "checkbox",
    "radio",
    "submit",
    "reset",
    "button",
    "hidden",
    "select-one",
    "select-multiple",
    "multiple",
    "textarea",
    "select",
];

/// Encodes events into the lowercase-hex `ce` blob: a header-less concatenation
/// of self-delimiting events. The event count and total size are carried by the
/// outer 2-byte length the payload assembler prepends (see
/// [`crate::token`]'s `inner_payload`), so there is no inner header.
pub fn encode(events: &[Event]) -> Result<String> {
    let mut out = Vec::new();
    for e in events {
        encode_event(e, &mut out)?;
    }
    Ok(hex::encode(out))
}

fn encode_event(e: &Event, out: &mut Vec<u8>) -> Result<()> {
    let handler = gb_handler_bytes(e.class);

    let mut flag = e.class & 0x3F;
    if handler.is_some() {
        flag |= 0x40;
    }
    if e.node_idx.is_some() {
        flag |= 0x80;
    }
    out.push(flag);

    if let Some(node_idx) = e.node_idx {
        let mut node = node_idx & 0x3F;
        if e.input_idx.is_some() {
            node |= 0x40;
        }
        if e.timestamp.is_some() {
            node |= 0x80;
        }
        out.push(node);
        if let Some(input_idx) = e.input_idx {
            out.push(input_idx);
        }
    }

    if let Some(len) = handler {
        if e.registry_bytes.len() != len {
            return Err(Error::BadRegistryLen {
                class: e.class,
                expected: len,
                got: e.registry_bytes.len(),
            });
        }
        out.extend_from_slice(&e.registry_bytes);
    }

    if let Some(ts) = e.timestamp {
        out.push(ts);
    }
    Ok(())
}

/// Decodes a header-less `ce` hex blob into its typed events, consuming the
/// whole buffer (events are self-delimiting; the outer length bounds the run).
pub fn decode(ce_hex: &str) -> Result<Vec<Event>> {
    let buf = hex::decode(ce_hex)?;
    let mut events = Vec::new();
    let mut cursor = 0;
    while cursor < buf.len() {
        let i = events.len();
        let (e, n) =
            decode_event(&buf[cursor..]).map_err(|m| Error::CeDecode(format!("event {i}: {m}")))?;
        events.push(e);
        cursor += n;
    }
    Ok(events)
}

fn decode_event(buf: &[u8]) -> std::result::Result<(Event, usize), String> {
    if buf.is_empty() {
        return Err("buffer exhausted".into());
    }
    let flag = buf[0];
    let mut cursor = 1;
    let class = flag & 0x3F;
    let has_handler = flag & 0x40 != 0;
    let has_node = flag & 0x80 != 0;

    let mut e = Event {
        class,
        ..Event::default()
    };

    let mut has_tc = false;
    if has_node {
        let node = *buf.get(cursor).ok_or("node byte runs off end")?;
        cursor += 1;
        e.node_idx = Some(node & 0x3F);
        let has_input = node & 0x40 != 0;
        has_tc = node & 0x80 != 0;
        if has_input {
            let ii = *buf.get(cursor).ok_or("input byte runs off end")?;
            cursor += 1;
            e.input_idx = Some(ii);
        }
    }

    if has_handler {
        let n = gb_handler_bytes(class)
            .ok_or_else(|| format!("unknown registry handler for class {class}"))?;
        if cursor + n > buf.len() {
            return Err("registry bytes run off end".into());
        }
        e.registry_bytes = buf[cursor..cursor + n].to_vec();
        cursor += n;
    }

    if has_tc {
        let ts = *buf.get(cursor).ok_or("timestamp byte runs off end")?;
        cursor += 1;
        e.timestamp = Some(ts);
    }
    Ok((e, cursor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_simple() {
        let events = vec![
            Event {
                class: 1,
                node_idx: Some(6),
                input_idx: Some(1),
                timestamp: Some(0x2a),
                registry_bytes: vec![],
            },
            Event {
                class: 19,
                registry_bytes: vec![0xab, 0xcd],
                ..Event::default()
            },
        ];
        let hex = encode(&events).unwrap();
        let decoded = decode(&hex).unwrap();
        assert_eq!(decoded, events);
    }
}
