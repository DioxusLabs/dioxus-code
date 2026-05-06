//! Capture precise edit ranges from a contenteditable's `beforeinput` event.
//!
//! Tree-sitter incremental parsing needs the byte range that changed. The
//! browser exposes that on the `beforeinput` event (target ranges + inserted
//! data) — but only *before* the DOM has been mutated, so by the time `oninput`
//! fires the ranges have collapsed. We install a raw `beforeinput` listener
//! via `web_sys`, build a [`SourceEdit`], and stash it for the upcoming
//! `oninput` to forward to the highlighter.

use std::{cell::RefCell, rc::Rc};

use dioxus_code::advanced::SourceEdit;
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{Element, Event, EventTarget, InputEvent, Node, Range, StaticRange};

/// Shared cell that the listener writes into and `CodeEditor::oninput` reads.
pub type PendingEdit = Rc<RefCell<Option<SourceEdit>>>;

/// Live `beforeinput` listener. Drop this to remove it.
pub struct InputEditCapture {
    closure: Closure<dyn FnMut(Event)>,
    target: EventTarget,
}

impl Drop for InputEditCapture {
    fn drop(&mut self) {
        let _ = self.target.remove_event_listener_with_callback(
            "beforeinput",
            self.closure.as_ref().unchecked_ref(),
        );
    }
}

/// Install a `beforeinput` listener on `element`.
///
/// The listener inspects each `InputEvent`, computes the byte range it will
/// edit relative to the element's text content, and writes a [`SourceEdit`]
/// into `pending`. Returns a handle that detaches the listener on drop.
pub fn install(element: Element, pending: PendingEdit) -> InputEditCapture {
    let editor_for_closure = element.clone();
    let closure = Closure::wrap(Box::new(move |event: Event| {
        let Ok(input_event) = event.dyn_into::<InputEvent>() else {
            return;
        };
        if let Some(edit) = compute_edit(&editor_for_closure, &input_event) {
            *pending.borrow_mut() = Some(edit);
        }
    }) as Box<dyn FnMut(Event)>);

    let target: EventTarget = element.into();
    target
        .add_event_listener_with_callback("beforeinput", closure.as_ref().unchecked_ref())
        .expect("beforeinput listener attached");

    InputEditCapture { closure, target }
}

fn compute_edit(editor: &Element, event: &InputEvent) -> Option<SourceEdit> {
    let ranges = event.get_target_ranges();
    if ranges.length() == 0 {
        return None;
    }
    let static_range: StaticRange = ranges.get(0).dyn_into().ok()?;
    let abstract_range: &web_sys::AbstractRange = static_range.unchecked_ref();

    let document = editor.owner_document()?;
    let editor_node: &Node = editor.unchecked_ref();
    let start_byte = byte_offset(
        &document,
        editor_node,
        &abstract_range.start_container(),
        abstract_range.start_offset(),
    )?;
    let old_end_byte = byte_offset(
        &document,
        editor_node,
        &abstract_range.end_container(),
        abstract_range.end_offset(),
    )?;

    let inserted_bytes = event.data().map(|d| d.len()).unwrap_or(0);
    Some(SourceEdit {
        start_byte,
        old_end_byte,
        new_end_byte: start_byte + inserted_bytes,
    })
}

/// Byte length of the textContent of `editor` from its start up to
/// `(container, offset)`. Implemented as `Range.toString().len()` since web-sys
/// doesn't bind `Range.prototype.toString` directly.
fn byte_offset(
    document: &web_sys::Document,
    editor: &Node,
    container: &Node,
    offset: u32,
) -> Option<usize> {
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(extends = "::js_sys::Object")]
        type RangeJs;
        #[wasm_bindgen(catch, method, js_name = toString)]
        fn to_string(this: &RangeJs) -> Result<String, JsValue>;
    }

    let range: Range = document.create_range().ok()?;
    range.set_start(editor, 0).ok()?;
    range.set_end(container, offset).ok()?;
    let range_js: &RangeJs = range.unchecked_ref();
    Some(range_js.to_string().ok()?.len())
}
