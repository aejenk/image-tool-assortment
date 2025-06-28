use std::{error::Error, fmt::Display, fs::File, io::Write};

pub struct SystemLog {
    log: File,
    app_log: File,
    indent: u8,
    indent_str: String,
    categories: Vec<String>,
    pause: bool,
}

type WriteResult<'a> = Result<&'a mut SystemLog, Box<dyn Error>>;

impl SystemLog {
    pub fn init(out: String) -> Result<SystemLog, Box<dyn Error>> {
        let log = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("{out}/log.log"))?;

        let app_log = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("{out}/app.log"))?;

        Ok(SystemLog {
            log,
            app_log,
            indent: 0,
            indent_str: "    ".into(),
            categories: vec![],
            pause: false,
        })
    }

    pub fn newline(&mut self) -> WriteResult {
        self.message("")
    }

    // State update

    pub fn indent(&mut self) -> &mut Self {
        self.indent = self.indent.saturating_add(1);
        self
    }

    pub fn unindent(&mut self) -> &mut Self {
        self.indent = self.indent.saturating_sub(1);
        self
    }

    // combo
    pub fn alert(&mut self, string: impl Display) -> WriteResult {
        self.message(format!("ALERT: {string}"))
    }

    // effects
    pub fn header(&mut self, string: impl Display) -> WriteResult {
        self.debug_log("header", format!("{string}"))?;
        writeln!(self.log, "[ {string:=^50} ]")?;
        writeln!(self.log, "[ {string:=^20} ]")?;
        Ok(self)
    }

    pub fn message(&mut self, string: impl Display) -> WriteResult {
        self.debug_log("message", format!("{string}"))?;
        if !self.pause {
            self.push_indent()?;
            writeln!(self.log, "{string}")?;
        }
        Ok(self)
    }

    pub fn pause(&mut self) -> &mut Self {
        self.pause = true;
        self
    }

    pub fn unpause(&mut self) -> &mut Self {
        self.pause = false;
        self
    }

    pub fn begin_category(&mut self, category: impl Display) -> WriteResult {
        self.indent();
        self.debug_log("category start", "")?;
        // self.message(format!("[{category}]"))?;
        self.message(format!("[ {:~^15} ]", format!(" {category} ")))?;
        self.categories.push(format!("{category}"));
        Ok(self)
    }

    pub fn state_property(&mut self, property: impl Display, value: impl Display) -> WriteResult {
        self.debug_log(
            "parsed property",
            format!("[{}.{property}]", self.categories.join(".")),
        )?;

        self.debug_log("data", format!("indent is [{}]", self.indent))?;

        //self.message(format!("({property}: {value}"))
        if self.indent != 0 {
            self.message(format!("|{property:_>15}: {value}"))
        } else {
            self.message(format!("{property:>15}: {value}"))
        }
    }

    pub fn end_category(&mut self) -> WriteResult {
        self.unindent();
        let exiting_category = self.categories.pop().unwrap_or("".to_string());
        self.debug_log("category end", format!("[/{exiting_category}]"))?;
        Ok(self)
    }

    // logs
    fn status_log(&mut self, logtype: &str, status: &str, string: impl Display) -> WriteResult {
        write!(self.app_log, "[{status:>5}] [{logtype:>20}]: ")?;
        writeln!(self.app_log, "{string}")?;
        Ok(self)
    }
    pub fn debug_log(&mut self, logtype: &str, string: impl Display) -> WriteResult {
        self.status_log(logtype, "DEBUG", string)
    }
    pub fn info_log(&mut self, logtype: &str, string: impl Display) -> WriteResult {
        self.status_log(logtype, "INFO", string)
    }
    pub fn warn_log(&mut self, logtype: &str, string: impl Display) -> WriteResult {
        self.status_log(logtype, "WARN", string)
    }
    pub fn error_log(&mut self, logtype: &str, string: impl Display) -> WriteResult {
        self.status_log(logtype, "ERROR", string)
    }
    pub fn fatal_log(&mut self, logtype: &str, string: impl Display) -> WriteResult {
        self.status_log(logtype, "FATAL", string)
    }

    pub fn sys_log(&mut self, string: impl Display) -> WriteResult {
        self.info_log("sys", string)
    }

    // utils
    fn push_indent(&mut self) -> WriteResult {
        for _ in 0..self.categories.len() {
            write!(self.log, "{}", self.indent_str)?;
        }
        Ok(self)
    }
}
