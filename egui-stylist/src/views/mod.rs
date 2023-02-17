//! This contains all the views that are used to construct the core of the application.
use std::{path::PathBuf, sync::Arc};

use egui::{CentralPanel, Checkbox, FontDefinitions, ScrollArea, SidePanel, Style, Ui, Widget};
use egui_theme::EguiTheme;
use serde::{Deserialize, Serialize};
mod colors;
mod fonts;
mod preview;
mod shape;
mod spacing;
mod text;

use preview::Preview;

use fonts::FontViewState;
use text::TextStyleViewState;

/// StylistFileDialogFunction is a function callback that allows the `StylistState` to open a native filedialog and get file paths for egui.
pub type StylistFileDialogFunction =
    Box<dyn Fn(StylistFileDialog, Option<(&str, &[&str])>) -> Option<PathBuf> + Send + Sync>;

pub type StylistOnClickFunction = Box<dyn Fn(&mut StylistState) + Send + Sync>;

/// This determines what kind of FileDialog is desired from within the `StylistState`
#[derive(PartialEq, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum StylistFileDialog {
    Open,
    Save,
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Copy, Debug)]
enum StylerTab {
    Colors,
    Fonts,
    TextStyles,
    Spacing,
    Shape,
}
/// This is the framework agnostic application state that can be easily embedded directly into any `egui` integration.
///
/// This can easily be embedded into any existing egui application by calling `ui` from within the egui context such as follows:
///
/// ```ignore
/// let state = StylistState::default():
/// egui::CentralPanel::default().show(ctx, |ui| state.ui(ui));
/// ```

#[derive(Serialize, Deserialize, Clone)]
pub struct StylistState {
    current_tab: StylerTab,
    show_stylist: bool,
    show_preview: bool,
    show_save_load_buttons: bool,
    style: Style,
    font_definitions: FontDefinitions,
    #[serde(skip)]
    font_view_state: FontViewState,
    #[serde(skip)]
    text_style_view_state: TextStyleViewState,
    preview: Preview,
    #[serde(skip)]
    pub file_dialog_function: Option<Arc<StylistFileDialogFunction>>,
    #[serde(skip)]
    on_save_function: Option<Arc<StylistOnClickFunction>>,
    #[serde(skip)]
    on_load_function: Option<Arc<StylistOnClickFunction>>,
}

impl PartialEq for StylistState {
    fn eq(&self, other: &Self) -> bool {
        self.current_tab == other.current_tab
            && self.show_stylist == other.show_stylist
            && self.show_preview == other.show_preview
            && self.show_save_load_buttons == other.show_save_load_buttons
            && self.style == other.style
            && self.font_definitions == other.font_definitions
            && self.font_view_state == other.font_view_state
            && self.text_style_view_state == other.text_style_view_state
            && self.preview == other.preview
    }
}

impl StylistState {
    pub fn default() -> Self {
        Self {
            current_tab: StylerTab::Colors,
            style: Style::default(),
            show_stylist: true,
            show_preview: true,
            show_save_load_buttons: false,
            font_definitions: FontDefinitions::default(),
            font_view_state: FontViewState::default(),
            text_style_view_state: TextStyleViewState::default(),
            preview: Preview::new(Style::default()),
            file_dialog_function: None,
            on_save_function: None,
            on_load_function: None,
        }
    }
    /// Sets `file_dialog_function` with the function call that it can use to
    pub fn set_file_dialog_function(&mut self, f: StylistFileDialogFunction) {
        self.file_dialog_function = Some(Arc::new(f));
    }
    pub fn show_save_load_buttons(&mut self, v: bool) {
        self.show_save_load_buttons = v;
    }

    pub fn set_on_save_function(&mut self, on_save_function: StylistOnClickFunction) {
        self.on_save_function = Some(Arc::new(on_save_function));
    }

    pub fn set_on_load_function(&mut self, on_load_function: StylistOnClickFunction) {
        self.on_load_function = Some(Arc::new(on_load_function));
    }
    /// Calls the file_dialog function and returns a path if it was found.
    pub fn file_dialog(
        &self,
        kind: StylistFileDialog,
        filter: Option<(&str, &[&str])>,
    ) -> Option<PathBuf> {
        self.file_dialog_function
            .as_ref()
            .and_then(|f| f(kind, filter))
    }

    fn tab_menu_ui(&mut self, ui: &mut Ui) {
        use egui::widgets::SelectableLabel;
        // Menu tabs
        ui.horizontal(|ui| {
            if ui
                .add(SelectableLabel::new(
                    self.current_tab == StylerTab::Colors,
                    "Colors",
                ))
                .clicked()
            {
                self.current_tab = StylerTab::Colors;
            }
            if ui
                .add(SelectableLabel::new(
                    self.current_tab == StylerTab::Fonts,
                    "Fonts",
                ))
                .clicked()
            {
                self.current_tab = StylerTab::Fonts;
            }
            if ui
                .add(SelectableLabel::new(
                    self.current_tab == StylerTab::TextStyles,
                    "TextStyles",
                ))
                .clicked()
            {
                self.current_tab = StylerTab::TextStyles;
            }

            if ui
                .add(SelectableLabel::new(
                    self.current_tab == StylerTab::Spacing,
                    "Spacing",
                ))
                .clicked()
            {
                self.current_tab = StylerTab::Spacing;
            }

            if ui
                .add(SelectableLabel::new(
                    self.current_tab == StylerTab::Shape,
                    "Shape",
                ))
                .clicked()
            {
                self.current_tab = StylerTab::Shape;
            }
            if self.show_save_load_buttons {
                if let Some(on_save) = &self.on_save_function {
                    if ui.button("Save").clicked() {
                        on_save(&mut self.clone());
                    }
                }
                if let Some(on_load) = &self.on_load_function {
                    if ui.button("Load").clicked() {
                        on_load(&mut self.clone());
                    }
                }
            }
            Checkbox::new(&mut self.show_stylist, "Show Stylist").ui(ui);
            Checkbox::new(&mut self.show_preview, "Show preview").ui(ui);
        });
    }
    /// Creates and displays the Stylist UI.
    /// This can be used to embed the Stylist into any application that supports it.
    pub fn ui(&mut self, ui: &mut Ui) {
        // Get the tab ui
        self.tab_menu_ui(ui);
        if self.show_stylist {
            SidePanel::left("_stylist_panel")
                .width_range(300.0..=900.0)
                .show_inside(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        // Show the content views.
                        match self.current_tab {
                            StylerTab::Colors => colors::colors_view(&mut self.style, ui),
                            StylerTab::Fonts => fonts::fonts_view(
                                &mut self.font_view_state,
                                self.file_dialog_function.as_ref(),
                                &mut self.font_definitions,
                                &mut self.style,
                                ui,
                            ),
                            StylerTab::TextStyles => {
                                let families = self
                                    .font_definitions
                                    .families
                                    .keys()
                                    .cloned()
                                    .collect::<Vec<_>>();
                                text::text_styles_view(
                                    &mut self.text_style_view_state,
                                    &mut self.style,
                                    families,
                                    ui,
                                )
                            }
                            StylerTab::Spacing => spacing::spacing_view(&mut self.style, ui),
                            StylerTab::Shape => shape::shape_view(&mut self.style, ui),
                        };
                    });
                });
        }
        if self.show_preview {
            CentralPanel::default().show_inside(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.preview.set_style(self.style.clone());
                    ui.ctx().set_fonts(self.font_definitions.clone());
                    self.preview.show(ui);
                });
            });
        }
    }
    pub fn export_theme(&self) -> EguiTheme {
        EguiTheme::new(self.style.clone(), self.font_definitions.clone())
    }
    pub fn import_theme(&mut self, theme: EguiTheme) {
        let (style, font_definitions) = theme.extract();
        self.style = style;
        self.font_definitions = font_definitions;
    }
}
