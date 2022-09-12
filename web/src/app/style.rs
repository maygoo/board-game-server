use egui::Color32;

pub struct Style { }

impl Style {
    // https://www.canva.com/colors/color-palettes/tropical-triumph/
    pub const OLIVE_GREEN: Color32      = Color32::from_rgb(68, 68, 64);
    pub const OLIVE_GREEN_DARK: Color32 = Color32::from_rgb(51, 51, 46);
    pub const LILAC: Color32            = Color32::from_rgb(195, 171, 192);
    pub const ROSE_QUARTZ: Color32      = Color32::from_rgb(240, 193, 176);
    pub const CORAL: Color32            = Color32::from_rgb(233, 147, 128);
    pub const CORAL_DARK: Color32       = Color32::from_rgb(255, 127, 80);

    pub fn build_style(mut style: egui::Style) -> egui::Style {
        // some notes on the colours in egui:
        // there is a text rendering bug that is 
        // more noticable when the background is
        // lighter than the text. See:
        // https://github.com/emilk/egui/pull/1765 &
        // https://github.com/emilk/egui/pull/1411
        // So we should probably pick a dark colour-scheme
    
        // https://www.canva.com/colors/color-palettes/salmon-sushi/
        /* let baby_blue = Color32::from_rgb(231, 242, 248);
        let aquamarine = Color32::from_rgb(116, 189, 203);
        let salmon = Color32::from_rgb(255, 163, 132);
        let freesia = Color32::from_rgb(239, 231, 188); */
    
        // this combo looked nice, but the text bug is too noticable
        // style.visuals.widgets.noninteractive.bg_fill = freesia;
        // style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, salmon);
    
        // https://www.canva.com/colors/color-palettes/padlocked-doors/
        /* let mauve = Color32::from_rgb(185, 144, 149);
        let salmon = Color32::from_rgb(252, 181, 172);
        let mint = Color32::from_rgb(181, 229, 207);
        let teal_green = Color32::from_rgb(61, 91, 89); */
    
        // https://www.canva.com/colors/color-palettes/ivy-swipe/
        /* let purple = Color32::from_rgb(94, 55, 109);
        let purple_dark = Color32::from_rgb(74, 43, 86);
        let lilac = Color32::from_rgb(189, 151, 203);
        let cream = Color32::from_rgb(243, 234, 192);
        let pewter = Color32::from_rgb(116, 112, 128);
        let pewter_dark = Color32::from_rgb(100, 95, 114); */
    
        // https://www.canva.com/colors/color-palettes/melted-ice-cream/
    
        // https://www.canva.com/colors/color-palettes/tropical-triumph/

        // darker background, e.g. text edits
        style.visuals.extreme_bg_color = Style::OLIVE_GREEN_DARK;
        // hyperlink colour
        style.visuals.hyperlink_color = Style::LILAC;
        // warning colour
        style.visuals.warn_fg_color = Style::CORAL;
    
        // visuals for windows
        // background colour
        style.visuals.widgets.noninteractive.bg_fill = Style::OLIVE_GREEN;
        // text colour
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, Style::ROSE_QUARTZ); // text colour
        // separator lines
        style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(2.0, Style::OLIVE_GREEN_DARK);
    
        // visuals for buttons at rest
        style.visuals.widgets.inactive.bg_fill = Style::CORAL;
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Style::OLIVE_GREEN);
    
        // visuals for buttons being hovered
        style.visuals.widgets.hovered.bg_fill = Style::CORAL_DARK;
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, Style::OLIVE_GREEN_DARK);
    
        // visuals for buttons being pressed
        style.visuals.widgets.active.bg_fill = Style::CORAL_DARK;
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, Style::OLIVE_GREEN_DARK);
    
        // font size
        let body = 18.0; // default 14
        let heading = 25.0; // default 20
    
        style.text_styles.insert(
            egui::style::TextStyle::Body,
            egui::FontId::new(body, egui::FontFamily::Proportional));
        style.text_styles.insert(
            egui::style::TextStyle::Heading,
            egui::FontId::new(heading, egui::FontFamily::Proportional));
    
        style
    }
}