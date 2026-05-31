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
    ///
    /// Note: serde serializes this `Vec<u8>` as a JSON number array. The bundled
    /// catalog ships typed events as `null`, so this never affects the wire
    /// output in practice.
    #[serde(default)]
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

/// Encodes events with the default header flag (0).
pub fn encode(events: &[Event]) -> Result<String> {
    encode_with_flag(events, 0)
}

/// Encodes events with a caller-supplied header flag. Returns lowercase hex.
pub fn encode_with_flag(events: &[Event], zb: u8) -> Result<String> {
    let mut out = vec![
        zb,
        ((events.len() >> 8) & 0xFF) as u8,
        (events.len() & 0xFF) as u8,
    ];
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

/// Decodes a `ce` hex blob back into the header flag and typed events.
pub fn decode(ce_hex: &str) -> Result<(u8, Vec<Event>)> {
    let buf = hex::decode(ce_hex)?;
    if buf.len() < 3 {
        return Err(Error::CeDecode("too short for header".into()));
    }
    let zb = buf[0];
    let count = ((buf[1] as usize) << 8) | buf[2] as usize;
    let mut cursor = 3;
    let mut events = Vec::with_capacity(count);
    for i in 0..count {
        let (e, n) =
            decode_event(&buf[cursor..]).map_err(|m| Error::CeDecode(format!("event {i}: {m}")))?;
        events.push(e);
        cursor += n;
    }
    if cursor != buf.len() {
        return Err(Error::CeDecode(format!(
            "{} trailing bytes after {} events",
            buf.len() - cursor,
            count
        )));
    }
    Ok((zb, events))
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
        let (zb, decoded) = decode(&hex).unwrap();
        assert_eq!(zb, 0);
        assert_eq!(decoded, events);
    }
}
