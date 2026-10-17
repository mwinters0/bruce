//! html5gum through its `CallbackEmitter`: borrowed slices, attributes
//! gathered into one reused buffer per start tag. The same stack as `gum`.

use crate::gum::Stack;
use crate::Raw;
use html5gum::emitters::callback::{CallbackEmitter, CallbackEvent};
use html5gum::{Span, Tokenizer};
use std::convert::Infallible;

pub fn extract(html: &str) -> Raw {
    let mut raw = Raw::default();
    let mut stack = Stack::default();
    let mut tag = String::new();
    // (name, value), first occurrence wins on lookup as the spec keeps the first.
    let mut attrs: Vec<(String, String)> = Vec::new();
    {
        let mut emitter = CallbackEmitter::new(|ev: CallbackEvent<'_>, _: Span<()>| -> Option<Infallible> {
            match ev {
                CallbackEvent::OpenStartTag { name } => {
                    tag.clear();
                    tag.push_str(&String::from_utf8_lossy(name));
                    attrs.clear();
                }
                CallbackEvent::AttributeName { name } => {
                    attrs.push((String::from_utf8_lossy(name).into_owned(), String::new()));
                }
                CallbackEvent::AttributeValue { value } => {
                    if let Some(last) = attrs.last_mut() {
                        last.1.push_str(&String::from_utf8_lossy(value));
                    }
                }
                CallbackEvent::CloseStartTag { self_closing } => {
                    let get = |n: &str| attrs.iter().find(|(k, _)| k == n).map(|(_, v)| v.clone());
                    stack.start(&mut raw, &tag, self_closing, &get);
                }
                CallbackEvent::EndTag { name } => stack.end(&String::from_utf8_lossy(name)),
                CallbackEvent::String { value } => stack.text(&mut raw, &String::from_utf8_lossy(value)),
                _ => {}
            }
            None
        });
        emitter.naively_switch_states(true);
        for r in Tokenizer::new_with_emitter(html, emitter) {
            r.unwrap();
        }
    }
    raw
}

/// Start tags, and nothing else: the tokenizer's own cost, for the bench.
pub fn count(html: &str) -> usize {
    let mut n = 0;
    {
        let mut emitter = CallbackEmitter::new(|ev: CallbackEvent<'_>, _: Span<()>| -> Option<Infallible> {
            if let CallbackEvent::OpenStartTag { .. } = ev {
                n += 1;
            }
            None
        });
        emitter.naively_switch_states(true);
        for r in Tokenizer::new_with_emitter(html, emitter) {
            r.unwrap();
        }
    }
    n
}
