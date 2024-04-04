use eframe::egui::{self, Button, RichText};
use egui::ImageButton;
use enum_map::{enum_map, Enum, EnumMap};

use crate::{
  PRIMARY_BUTTON_BACKGROUND_COLOR,
  PRIMARY_BUTTON_TEXT_COLOR, SECONDARY_BUTTON_TEXT_COLOR, SECONDARY_BUTTON_BACKGROUND_COLOR
};

#[derive(Debug, Enum, Clone, Copy)]
pub enum ButtonVariant {
  Primary,
  Secondary,
}

pub fn button(text: &str, variant: ButtonVariant) -> Button<'_> {
  let text_color = enum_map! {
    ButtonVariant::Primary => PRIMARY_BUTTON_TEXT_COLOR,
    ButtonVariant::Secondary => SECONDARY_BUTTON_TEXT_COLOR
  };

  let background_color: EnumMap<ButtonVariant, egui::Color32> = enum_map! {
    ButtonVariant::Primary => PRIMARY_BUTTON_BACKGROUND_COLOR,
    ButtonVariant::Secondary => SECONDARY_BUTTON_BACKGROUND_COLOR
  };

  egui::Button::new(RichText::new(text).color(text_color[variant])).frame(false).fill(background_color[variant])
}

pub fn icon_button<'a>(icon: egui::ImageSource<'a>, text: &'a str, variant: ButtonVariant) -> Button<'a> {
  let text_color = enum_map! {
    ButtonVariant::Primary => PRIMARY_BUTTON_TEXT_COLOR,
    ButtonVariant::Secondary => SECONDARY_BUTTON_TEXT_COLOR
  };

  let background_color: EnumMap<ButtonVariant, egui::Color32> = enum_map! {
    ButtonVariant::Primary => PRIMARY_BUTTON_BACKGROUND_COLOR,
    ButtonVariant::Secondary => SECONDARY_BUTTON_BACKGROUND_COLOR
  };

  egui::Button::image_and_text(icon, RichText::new(text).color(text_color[variant])).fill(background_color[variant])
}