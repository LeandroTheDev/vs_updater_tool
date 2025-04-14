use colored::*;

pub struct LogsInstance {}

impl LogsInstance {
    pub fn print(log: &str, color: Color) {
        let formated_color: ColoredString;
        match color {
            Color::Black => formated_color = log.black(),
            Color::Red => formated_color = log.red(),
            Color::Green => formated_color = log.green(),
            Color::Yellow => formated_color = log.yellow(),
            Color::Blue => formated_color = log.blue(),
            Color::Magenta => formated_color = log.magenta(),
            Color::Cyan => formated_color = log.cyan(),
            Color::White => formated_color = log.white(),
            Color::BrightBlack => formated_color = log.bright_black(),
            Color::BrightRed => formated_color = log.bright_red(),
            Color::BrightGreen => formated_color = log.bright_green(),
            Color::BrightYellow => formated_color = log.bright_yellow(),
            Color::BrightBlue => formated_color = log.bright_blue(),
            Color::BrightMagenta => formated_color = log.bright_magenta(),
            Color::BrightCyan => formated_color = log.bright_cyan(),
            Color::BrightWhite => formated_color = log.bright_white(),
            // Default
            _ => formated_color = log.white(),
        }
        println!("{}", formated_color);
    }
}