use std::fmt::{Display, Formatter};

// logging structs
pub struct AppLog {
    runs: Vec<RunLog>,
}
pub struct RunLog {
    entries: Vec<LogEntry>,
    iteration: usize,
}

pub struct LogEntry {
    message: String,
    nesting_level: usize
}

// LOGIC
impl AppLog {
    pub fn init() -> Self {
        AppLog { runs: vec![] }
    }

    pub fn add_run(&mut self, run: RunLog) -> &mut Self {
        self.runs.push(run);
        self
    }
}

impl RunLog {
    pub fn init(iteration: usize) -> Self {
        RunLog { entries: vec![], iteration }
    }

    fn add_entry(&mut self, entry: LogEntry) -> &mut Self {
        self.entries.push(entry);
        self
    }

    pub fn apply_effect(&mut self, name: &'static str, parameters: Vec<(&'static str, String)>) -> &mut Self {
        let mut entry = LogEntry::init();

        let mut message = format!(
            "Applying effect [{}], with the following parameters...\n", name
        );

        entry.tab_in();

        for (p_name, value) in parameters {
            message.push_str(&entry.tabs(format!("[{name}.{p_name}]: {value}\n")));
        }

        entry.message = message;

        self.add_entry(entry)
    }
}

impl LogEntry {
    pub fn init() -> LogEntry {
        LogEntry {
            message: String::new(),
            nesting_level: 1,
        }
    }

    fn tabs(&self, message: String) -> String {
        format!("{}{}", self.get_tabs(), message)
    } 

    fn tab_in(&mut self) -> &mut Self {
        self.nesting_level = self.nesting_level + 1;
        self
    }

    // utils
    fn get_tabs(&self) -> String {
        let mut tabs = String::new();

        for _ in 0..self.nesting_level {
            tabs.push_str("\t");
        }

        tabs
    }
}

// display impls
impl Display for AppLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ === RUNS === ]\n{}", join_to_string(&self.runs, "\n\n"))
    }
}

impl Display for RunLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:05}]\n{}", &self.iteration, join_to_string(&self.entries, "\n"))
    }
}

impl Display for LogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// misc utils
fn join_to_string<T: Display>(vec: &Vec<T>, delimiter: &'static str) -> String {
    vec
    .iter()
    .map(|s| format!("{}", s))
    .collect::<Vec<String>>()
    .join(delimiter)
}