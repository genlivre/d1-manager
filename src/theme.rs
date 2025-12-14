#![allow(dead_code)]

use eframe::egui::{self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, FontTweak, Stroke, Vec2, Visuals};

/// App color palette - Modern, minimal design
pub struct AppColors;

impl AppColors {
    // Primary colors
    pub const PRIMARY: Color32 = Color32::from_rgb(59, 130, 246);      // Blue-500
    pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(37, 99, 235); // Blue-600
    pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(219, 234, 254); // Blue-100

    // Semantic colors
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);       // Green-500
    pub const WARNING: Color32 = Color32::from_rgb(245, 158, 11);      // Amber-500
    pub const ERROR: Color32 = Color32::from_rgb(239, 68, 68);         // Red-500

    // Neutral colors (Dark theme)
    pub const BG_PRIMARY: Color32 = Color32::from_rgb(17, 24, 39);     // Gray-900
    pub const BG_SECONDARY: Color32 = Color32::from_rgb(31, 41, 55);   // Gray-800
    pub const BG_TERTIARY: Color32 = Color32::from_rgb(55, 65, 81);    // Gray-700
    pub const BG_HOVER: Color32 = Color32::from_rgb(75, 85, 99);       // Gray-600

    // Text colors
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(249, 250, 251);   // Gray-50
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(156, 163, 175); // Gray-400
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(107, 114, 128);     // Gray-500

    // Border colors
    pub const BORDER: Color32 = Color32::from_rgb(55, 65, 81);         // Gray-700
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(75, 85, 99);   // Gray-600
}

/// Spacing constants (8px grid system)
pub struct Spacing;

impl Spacing {
    pub const NONE: f32 = 0.0;
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const XL: f32 = 32.0;
    pub const XXL: f32 = 48.0;
}

/// Border radius constants
pub struct Radius;

impl Radius {
    pub const NONE: CornerRadius = CornerRadius::ZERO;
    pub const XS: CornerRadius = CornerRadius::same(2);
    pub const SM: CornerRadius = CornerRadius::same(4);
    pub const MD: CornerRadius = CornerRadius::same(6);
    pub const LG: CornerRadius = CornerRadius::same(8);
    pub const FULL: CornerRadius = CornerRadius::same(255);
}

/// Configure fonts with Japanese support
fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // System font paths for Japanese support
    #[cfg(target_os = "macos")]
    let japanese_font_paths = [
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/ヒラギノ丸ゴ ProN W4.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
    ];

    #[cfg(target_os = "windows")]
    let japanese_font_paths = [
        "C:\\Windows\\Fonts\\msgothic.ttc",
        "C:\\Windows\\Fonts\\meiryo.ttc",
        "C:\\Windows\\Fonts\\YuGothM.ttc",
        "C:\\Windows\\Fonts\\yugothic.ttf",
    ];

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let japanese_font_paths: [&str; 0] = [];

    // Try to load Japanese font
    for path in japanese_font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            // Apply tweak to adjust Japanese font metrics to match English fonts
            let mut font = FontData::from_owned(font_data);
            font.tweak = FontTweak {
                scale: 1.0,
                y_offset_factor: 0.2,  // Move glyphs up to align with English baseline
                y_offset: 0.0,
            };

            fonts.font_data.insert(
                "japanese".to_owned(),
                font.into(),
            );

            // Add Japanese font as fallback for all font families
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push("japanese".to_owned());

            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push("japanese".to_owned());

            break;
        }
    }

    ctx.set_fonts(fonts);
}

/// Configure the app's visual theme
pub fn configure_theme(ctx: &egui::Context) {
    // Configure Japanese font support
    configure_fonts(ctx);

    let mut style = (*ctx.style()).clone();

    // Spacing
    style.spacing.item_spacing = Vec2::new(Spacing::SM, Spacing::SM);
    style.spacing.window_margin = egui::Margin::same(Spacing::MD as i8);
    style.spacing.button_padding = Vec2::new(Spacing::MD, Spacing::SM);
    style.spacing.indent = Spacing::MD;

    // Text styles
    style.text_styles.insert(
        egui::TextStyle::Heading,
        FontId::new(20.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        FontId::new(13.0, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        FontId::new(12.0, egui::FontFamily::Proportional),
    );

    ctx.set_style(style);

    // Dark theme visuals
    let mut visuals = Visuals::dark();

    // Window/panel backgrounds
    visuals.window_fill = AppColors::BG_PRIMARY;
    visuals.panel_fill = AppColors::BG_PRIMARY;
    visuals.faint_bg_color = AppColors::BG_SECONDARY;
    visuals.extreme_bg_color = AppColors::BG_TERTIARY;

    // Widget styling
    visuals.widgets.noninteractive.bg_fill = AppColors::BG_SECONDARY;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, AppColors::TEXT_SECONDARY);
    visuals.widgets.noninteractive.corner_radius = Radius::MD;

    visuals.widgets.inactive.bg_fill = AppColors::BG_SECONDARY;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, AppColors::TEXT_PRIMARY);
    visuals.widgets.inactive.corner_radius = Radius::MD;

    visuals.widgets.hovered.bg_fill = AppColors::BG_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, AppColors::TEXT_PRIMARY);
    visuals.widgets.hovered.corner_radius = Radius::MD;

    visuals.widgets.active.bg_fill = AppColors::PRIMARY;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.corner_radius = Radius::MD;

    // Selection
    visuals.selection.bg_fill = AppColors::PRIMARY.gamma_multiply(0.3);
    visuals.selection.stroke = Stroke::new(1.0, AppColors::PRIMARY);

    // Window styling
    visuals.window_stroke = Stroke::new(1.0, AppColors::BORDER);

    // Separator
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, AppColors::BORDER);

    ctx.set_visuals(visuals);
}

/// Custom styled button
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text).color(Color32::WHITE)
    )
    .fill(AppColors::PRIMARY)
    .corner_radius(Radius::MD);

    ui.add(button)
}

/// Secondary button (outlined style)
pub fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text).color(AppColors::TEXT_PRIMARY)
    )
    .fill(Color32::TRANSPARENT)
    .stroke(Stroke::new(1.0, AppColors::BORDER_LIGHT))
    .corner_radius(Radius::MD);

    ui.add(button)
}

/// Danger button (red)
pub fn danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text).color(Color32::WHITE)
    )
    .fill(AppColors::ERROR)
    .corner_radius(Radius::MD);

    ui.add(button)
}

/// Section heading with consistent styling
pub fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(Spacing::SM);
    ui.label(
        egui::RichText::new(text)
            .size(18.0)
            .color(AppColors::TEXT_PRIMARY)
            .strong()
    );
    ui.add_space(Spacing::MD);
}

/// Muted/secondary text
pub fn muted_text(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(12.0)
            .color(AppColors::TEXT_MUTED)
    );
}

/// Status indicator dot
pub fn status_dot(ui: &mut egui::Ui, connected: bool) {
    let color = if connected {
        AppColors::SUCCESS
    } else {
        AppColors::ERROR
    };
    ui.label(egui::RichText::new("●").color(color).size(10.0));
}

/// Card-like container
pub fn card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(AppColors::BG_SECONDARY)
        .corner_radius(Radius::LG)
        .inner_margin(egui::Margin::same(Spacing::MD as i8))
        .stroke(Stroke::new(1.0, AppColors::BORDER))
        .show(ui, add_contents);
}

/// Input field with label
pub fn labeled_input(ui: &mut egui::Ui, label: &str, value: &mut String, password: bool) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(label)
                .size(13.0)
                .color(AppColors::TEXT_SECONDARY)
        );
        ui.add_space(Spacing::XS);

        let mut text_edit = egui::TextEdit::singleline(value)
            .desired_width(f32::INFINITY)
            .margin(egui::Margin::symmetric(Spacing::SM as i8, Spacing::SM as i8));

        if password {
            text_edit = text_edit.password(true);
        }

        ui.add(text_edit);
    });
}

/// Tab button styling
pub fn tab_button(ui: &mut egui::Ui, label: &str, active: bool, connected: bool) -> egui::Response {
    let text_color = if active {
        AppColors::TEXT_PRIMARY
    } else {
        AppColors::TEXT_SECONDARY
    };

    let bg_color = if active {
        AppColors::BG_TERTIARY
    } else {
        Color32::TRANSPARENT
    };

    let status_indicator = if connected { "● " } else { "○ " };
    let status_color = if connected {
        AppColors::SUCCESS
    } else {
        AppColors::TEXT_MUTED
    };

    let response = ui.horizontal(|ui| {
        ui.label(egui::RichText::new(status_indicator).color(status_color).size(10.0));
        ui.label(egui::RichText::new(label).color(text_color));
    });

    // Draw background
    if active {
        ui.painter().rect_filled(
            response.response.rect.expand(2.0),
            Radius::SM,
            bg_color,
        );
    }

    response.response
}
