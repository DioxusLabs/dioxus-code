//! Track precise edit ranges for incremental highlighting.
//!
//! Tree-sitter incremental parsing needs the byte range that changed. The
//! browser exposes the textarea selection range on the `beforeinput` event -
//! but only *before* the DOM has been mutated, so by the time `oninput` fires
//! the selection has moved. On web targets this module installs a raw
//! `beforeinput` listener via `web_sys`, builds a [`SourceEdit`], and stashes
//! it for the next render. On desktop, it diffs the last highlighted value
//! against the latest input value.

use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use dioxus_code::advanced::SourceEdit;

#[cfg(target_arch = "wasm32")]
#[path = "edit_capture/web.rs"]
mod platform;
#[cfg(not(target_arch = "wasm32"))]
#[path = "edit_capture/desktop.rs"]
mod platform;

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingSourceEdit {
    edit: SourceEdit,
    new_value: String,
}

/// Cross-platform source edit tracker used by `CodeEditor`.
pub struct InputEditTracker {
    rendered_value: String,
    latest_value: String,
    pending: Option<PendingSourceEdit>,
    platform: platform::PlatformEditTracker,
}

impl InputEditTracker {
    /// Create a tracker with no pending edit.
    pub fn new(value: String) -> Self {
        Self {
            rendered_value: value.clone(),
            latest_value: value,
            pending: None,
            platform: platform::PlatformEditTracker::new(),
        }
    }

    /// Mount platform-specific edit capture for the editor input element.
    pub fn mount(&mut self, event: MountedEvent) {
        self.platform.mount(event);
    }

    /// Track a new full input value and produce a pending incremental edit.
    pub fn input(&mut self, value: String) {
        let edit = self
            .platform
            .input(&self.rendered_value, &self.latest_value, &value);

        self.pending = edit.map(|edit| PendingSourceEdit {
            edit,
            new_value: value.clone(),
        });
        self.latest_value = value;
    }

    /// Take the pending edit for the source about to be highlighted.
    pub fn take_for_render(&mut self, value: &str) -> Option<SourceEdit> {
        if value == self.rendered_value {
            return None;
        }

        let edit = self
            .pending
            .take()
            .filter(|pending| pending.new_value == value)
            .map(|pending| pending.edit);

        self.rendered_value = value.to_string();
        self.latest_value = value.to_string();
        self.platform.clear();

        edit
    }
}

/// Build the input event attributes for the editor input element.
pub fn use_input_edit_attributes(
    tracker: Rc<RefCell<InputEditTracker>>,
    mut on_value: impl FnMut(String) + 'static,
) -> Vec<Attribute> {
    let mount_tracker = tracker.clone();
    vec![
        onmounted(move |event: MountedEvent| {
            mount_tracker.borrow_mut().mount(event);
        }),
        oninput(move |event: FormEvent| {
            let value = event.value();
            tracker.borrow_mut().input(value.clone());
            on_value(value);
        }),
    ]
}

#[cfg(any(target_arch = "wasm32", test))]
fn utf16_offset_to_byte_offset(value: &str, offset: u32) -> Option<usize> {
    if offset == 0 {
        return Some(0);
    }

    let mut utf16_len = 0;
    for (byte_offset, ch) in value.char_indices() {
        if utf16_len == offset {
            return Some(byte_offset);
        }
        utf16_len += ch.len_utf16() as u32;
        if utf16_len > offset {
            return None;
        }
    }

    (utf16_len == offset).then_some(value.len())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn render_takes_diff_from_rendered_baseline_to_latest_input() {
        let mut tracker = InputEditTracker::new("abc".to_string());

        tracker.input("abx".to_string());
        tracker.input("abxy".to_string());

        assert_eq!(
            tracker.take_for_render("abxy"),
            Some(SourceEdit {
                start_byte: 2,
                old_end_byte: 3,
                new_end_byte: 4,
            })
        );
    }

    #[test]
    fn external_value_change_drops_pending_edit() {
        let mut tracker = InputEditTracker::new("abc".to_string());

        tracker.input("abx".to_string());

        assert_eq!(tracker.take_for_render("xyz"), None);
    }

    #[test]
    fn utf16_offsets_convert_to_byte_offsets() {
        let value = "a💡é";

        assert_eq!(utf16_offset_to_byte_offset(value, 0), Some(0));
        assert_eq!(utf16_offset_to_byte_offset(value, 1), Some(1));
        assert_eq!(utf16_offset_to_byte_offset(value, 3), Some(5));
        assert_eq!(utf16_offset_to_byte_offset(value, 4), Some(7));
    }

    #[test]
    fn utf16_offsets_reject_surrogate_boundaries() {
        assert_eq!(utf16_offset_to_byte_offset("a💡", 2), None);
    }
}
