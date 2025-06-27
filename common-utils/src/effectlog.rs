use std::ops::AddAssign;

use palette::rgb::Rgb;

pub enum LogEntry {
    Effect {
        name: String,
        options: String,
    },
    AddColourToPalette {
        colour: Rgb,
    },
    AddGradientToPalette {
        colour: Rgb,
        shades: u16,
    }
}

impl std::fmt::Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogEntry::Effect { name, options } => {
                write!(f, "Effect [{name}] with options [{options}]")
            },
            LogEntry::AddColourToPalette { colour } => {
                let (r, g, b) = colour.into_components();
                write!(f, "Rgb::new({r:.5}, {g:.5}, {b:.5})")
            },
            LogEntry::AddGradientToPalette { colour, shades } => {
                let (r, g, b) = colour.into_components();
                write!(f, "Rgb::new({r:.5}, {g:.5}, {b:.5}).build_gradient_lch({shades})")
            }
        }
    }
}

impl LogEntry {
    pub fn effect(name: String, options: String) -> Self {
        Self::Effect { name, options }
    }

    pub fn colour(colour: Rgb) -> Self {
        Self::AddColourToPalette { colour }
    }

    pub fn gradient(colour: Rgb, shades: u16) -> Self {
        Self::AddGradientToPalette { colour, shades }
    }
}

pub struct ExecLog {
    log: Vec<LogEntry>,
}

impl Default for ExecLog {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecLog {
    pub fn new() -> Self {
        ExecLog { log: Vec::new() }
    }

    pub fn add_entry(&mut self, entry: LogEntry) -> &mut Self {
        self.log.push(entry);
        self
    }

    pub fn write_to(&self, name: &str) -> std::io::Result<()> {
        std::fs::write(
            name,
            self.log.iter()
                .map(|entry| entry.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    pub fn reset(self) -> Self {
        Self::new()
    }
}

impl AddAssign for ExecLog {
    fn add_assign(&mut self, rhs: Self) {
        for entry in rhs.log {
            self.add_entry(entry);
        }
    }
}