use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::MountedEvent;
use dioxus_code::advanced::SourceEdit;
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::{Event, EventTarget, HtmlTextAreaElement, InputEvent};

use super::utf16_offset_to_byte_offset;

type PendingEdit = Rc<RefCell<Option<SourceEdit>>>;

pub struct PlatformEditTracker {
    captured_edit: PendingEdit,
    capture: Option<BeforeInputCapture>,
}

impl PlatformEditTracker {
    pub fn new() -> Self {
        Self {
            captured_edit: Rc::new(RefCell::new(None)),
            capture: None,
        }
    }

    pub fn mount(&mut self, event: MountedEvent) {
        use dioxus::web::WebEventExt;

        if let Some(element) = event.try_as_web_event() {
            if let Ok(textarea) = element.dyn_into::<HtmlTextAreaElement>() {
                self.capture = Some(BeforeInputCapture::install(
                    textarea,
                    self.captured_edit.clone(),
                ));
            }
        }
    }

    pub fn input(
        &mut self,
        rendered_value: &str,
        latest_value: &str,
        _value: &str,
    ) -> Option<SourceEdit> {
        self.captured_edit
            .borrow_mut()
            .take()
            .filter(|_| latest_value == rendered_value)
    }

    pub fn clear(&mut self) {
        self.captured_edit.borrow_mut().take();
    }
}

struct BeforeInputCapture {
    closure: Closure<dyn FnMut(Event)>,
    target: EventTarget,
}

impl Drop for BeforeInputCapture {
    fn drop(&mut self) {
        let _ = self.target.remove_event_listener_with_callback(
            "beforeinput",
            self.closure.as_ref().unchecked_ref(),
        );
    }
}

impl BeforeInputCapture {
    fn install(textarea: HtmlTextAreaElement, pending: PendingEdit) -> Self {
        let textarea_for_closure = textarea.clone();
        let closure = Closure::wrap(Box::new(move |event: Event| {
            let Ok(input_event) = event.dyn_into::<InputEvent>() else {
                return;
            };
            if let Some(edit) = source_edit_from_beforeinput(&textarea_for_closure, &input_event) {
                *pending.borrow_mut() = Some(edit);
            }
        }) as Box<dyn FnMut(Event)>);

        let target: EventTarget = textarea.into();
        target
            .add_event_listener_with_callback("beforeinput", closure.as_ref().unchecked_ref())
            .expect("beforeinput listener attached");

        Self { closure, target }
    }
}

fn source_edit_from_beforeinput(
    textarea: &HtmlTextAreaElement,
    event: &InputEvent,
) -> Option<SourceEdit> {
    let value = textarea.value();
    let start = textarea.selection_start().ok()??;
    let end = textarea.selection_end().ok()??;
    let start_byte = utf16_offset_to_byte_offset(&value, start)?;
    let old_end_byte = utf16_offset_to_byte_offset(&value, end)?;
    let inserted_bytes = match event.data() {
        Some(data) => data.len(),
        None if old_end_byte > start_byte || event.input_type().starts_with("delete") => 0,
        None => return None,
    };

    Some(SourceEdit {
        start_byte,
        old_end_byte,
        new_end_byte: start_byte + inserted_bytes,
    })
}
