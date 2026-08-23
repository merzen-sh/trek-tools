use clap::builder::styling::{AnsiColor, Color, Effects, Style, Styles};

pub fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
        .valid(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .invalid(AnsiColor::Yellow.on_default() | Effects::BOLD)
}

const DESC: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::White)));

pub fn desc(text: &str) -> String {
    format!("{DESC}{text}{DESC:#}")
}
