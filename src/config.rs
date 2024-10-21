use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct PartialTheme {
    fg_connected_color: Option<String>,
    fg_header_color: Option<String>,
    fg_selected_color: Option<String>,
    fg_normal_color: Option<String>,
    fg_new_device_color: Option<String>,

    bg_connected_color: Option<String>,
    bg_header_color: Option<String>,
    bg_selected_color: Option<String>,
    bg_normal_color: Option<String>,
    bg_new_device_color: Option<String>,

    column_spacing: Option<u16>,
    border_color: Option<String>,
    borders: Option<bool>,
    rounded_borders: Option<bool>,
    scrollbars: Option<bool>,
    date_format: Option<String>,
}
#[derive(Deserialize)]
pub struct Theme {
    pub fg_connected_color: String,
    pub fg_header_color: String,
    pub fg_selected_color: String,
    pub fg_normal_color: String,
    pub fg_new_device_color: String,

    pub bg_connected_color: String,
    pub bg_header_color: String,
    pub bg_selected_color: String,
    pub bg_normal_color: String,
    pub bg_new_device_color: String,

    pub column_spacing: u16,
    pub border_color: String,
    pub borders: bool,
    pub rounded_borders: bool,
    pub scrollbars: bool,
    pub date_format: String,
}
impl From<PartialTheme> for Theme {
    fn from(val: PartialTheme) -> Self {
        Self {
            fg_connected_color: val.fg_connected_color.unwrap_or("lightgreen".to_string()),
            fg_header_color: val.fg_header_color.unwrap_or("cyan".to_string()),
            fg_selected_color: val.fg_selected_color.unwrap_or("white".to_string()),
            fg_normal_color: val.fg_normal_color.unwrap_or("white".to_string()),
            fg_new_device_color: val.fg_new_device_color.unwrap_or("yellow".to_string()),

            bg_connected_color: val.bg_connected_color.unwrap_or("black".to_string()),
            bg_header_color: val.bg_header_color.unwrap_or("black".to_string()),
            bg_selected_color: val.bg_selected_color.unwrap_or("darkgray".to_string()),
            bg_normal_color: val.bg_normal_color.unwrap_or("black".to_string()),
            bg_new_device_color: val.bg_new_device_color.unwrap_or("black".to_string()),

            border_color: val.border_color.unwrap_or("blue".to_string()),
            borders: val.borders.unwrap_or(true),
            rounded_borders: val.rounded_borders.unwrap_or(false),
            date_format: val.date_format.unwrap_or_else(|| "%Y-%m-%d".to_string()),
            scrollbars: val.scrollbars.unwrap_or(false),
            column_spacing: val.column_spacing.unwrap_or(4),
        }
    }
}
impl Default for Theme {
    fn default() -> Self {
        Self::from(PartialTheme::default())
    }
}

#[derive(Deserialize, Default)]
pub struct PartialConfig {
    theme: Option<PartialTheme>,
}
#[derive(Deserialize, Default)]
pub struct Config {
    pub theme: Theme,
}
impl From<PartialConfig> for Config {
    fn from(val: PartialConfig) -> Self {
        Self {
            theme: Theme::from(val.theme.unwrap_or_default()),
        }
    }
}
