use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::stdout;

#[derive(Debug, PartialEq, Clone)]
pub struct Logger {
    pub prefix: String,
    pub color: Color,
    pub width: usize,
}

pub struct Message {
    pub text: String,
    pub color: Color,
}

impl Logger {
    pub fn new(prefix: String, color: Color) -> Self {
        Logger {
            prefix,
            color,
            width: 30,
        }
    }

    pub fn log(&self, message: Message) -> Result<(), std::io::Error> {
        let prefix = format!("[{}]", self.prefix);
        let formatted_prefix = format!("{:>width$}", prefix, width = self.width - 2);
        let formatted_message = format!(" {}\n", message.text);

        execute!(
            stdout(),
            SetForegroundColor(self.color),
            Print(formatted_prefix),
            ResetColor
        )?;

        execute!(
            stdout(),
            SetForegroundColor(message.color),
            Print(formatted_message),
            ResetColor
        )
    }
}
