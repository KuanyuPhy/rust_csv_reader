use egui::{Context, FontDefinitions, FontFamily};

pub fn setup_custom_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    // Try to load system fonts that support Chinese characters
    #[cfg(target_os = "windows")]
    {
        try_load_windows_fonts(&mut fonts);
    }

    // Always set fonts (even if no Chinese font was loaded, this ensures proper Unicode handling)
    ctx.set_fonts(fonts);
}

#[cfg(target_os = "windows")]
fn try_load_windows_fonts(fonts: &mut FontDefinitions) -> bool {
    let font_paths = [
        "C:/Windows/Fonts/msyh.ttc",   // Microsoft YaHei
        "C:/Windows/Fonts/simsun.ttc", // SimSun
        "C:/Windows/Fonts/msyhl.ttc",  // Microsoft YaHei Light
    ];

    for font_path in &font_paths {
        if let Ok(font_data) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "chinese_font".to_owned(),
                egui::FontData::from_owned(font_data),
            );

            // Add the font to the font families
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese_font".to_owned());
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "chinese_font".to_owned());

            return true;
        }
    }

    false
}
