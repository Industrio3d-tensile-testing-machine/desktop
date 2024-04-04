use eframe::egui::{self, RichText, SelectableLabel};

pub fn selectable_label(selected: bool, text: &str) -> SelectableLabel {
  egui::SelectableLabel::new(selected,  RichText::new(text))
}