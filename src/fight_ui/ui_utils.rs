use bevy_inspector_egui::egui::{Response, Ui, WidgetText};

pub fn un_selectable_value<Value: PartialEq>(
    ui: &mut Ui,
    current_value: &mut Option<Value>,
    selected_value: Value,
    text: impl Into<WidgetText>,
) -> Response {
    let was_selected: bool = current_value.as_ref() == Some(&selected_value);

    let mut response = ui.selectable_label(was_selected, text);
    if response.clicked() {
        *current_value = if was_selected {
            None
        } else {
            Some(selected_value)
        };
        response.mark_changed();
    }
    response
}
