use dioxus::prelude::MountedEvent;
use dioxus_code::advanced::SourceEdit;

pub struct PlatformEditTracker;

impl PlatformEditTracker {
    pub fn new() -> Self {
        Self
    }

    pub fn mount(&mut self, event: MountedEvent) {
        let _ = event;
    }

    pub fn input(
        &mut self,
        rendered_value: &str,
        _latest_value: &str,
        value: &str,
    ) -> Option<SourceEdit> {
        source_edit_from_diff(rendered_value, value)
    }

    pub fn clear(&mut self) {}
}

pub(super) fn source_edit_from_diff(old: &str, new: &str) -> Option<SourceEdit> {
    if old == new {
        return None;
    }

    let old_bytes = old.as_bytes();
    let new_bytes = new.as_bytes();
    let mut start = 0;
    let shared_len = old.len().min(new.len());

    while start < shared_len && old_bytes[start] == new_bytes[start] {
        start += 1;
    }
    while start > 0 && (!old.is_char_boundary(start) || !new.is_char_boundary(start)) {
        start -= 1;
    }

    let mut old_end = old.len();
    let mut new_end = new.len();
    while old_end > start && new_end > start && old_bytes[old_end - 1] == new_bytes[new_end - 1] {
        old_end -= 1;
        new_end -= 1;
    }
    while old_end < old.len() && !old.is_char_boundary(old_end) {
        old_end += 1;
    }
    while new_end < new.len() && !new.is_char_boundary(new_end) {
        new_end += 1;
    }

    Some(SourceEdit {
        start_byte: start,
        old_end_byte: old_end,
        new_end_byte: new_end,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_tracks_insertion() {
        assert_eq!(
            source_edit_from_diff("abc", "abxc"),
            Some(SourceEdit {
                start_byte: 2,
                old_end_byte: 2,
                new_end_byte: 3,
            })
        );
    }

    #[test]
    fn diff_tracks_deletion() {
        assert_eq!(
            source_edit_from_diff("abxc", "abc"),
            Some(SourceEdit {
                start_byte: 2,
                old_end_byte: 3,
                new_end_byte: 2,
            })
        );
    }

    #[test]
    fn diff_tracks_unicode_replacement_on_char_boundaries() {
        assert_eq!(
            source_edit_from_diff("aéz", "aèz"),
            Some(SourceEdit {
                start_byte: 1,
                old_end_byte: 3,
                new_end_byte: 3,
            })
        );
    }
}
