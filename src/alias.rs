use std::fs;
use std::path::Path;

use crate::mode::MatchResult;

pub struct AliasWriter {
    prefix: String,
    fmt_string: String,
    buf: String,
}

impl AliasWriter {
    pub fn new(prefix: String, fmt_string: String) -> Self {
        Self {
            prefix,
            fmt_string,
            buf: String::new(),
        }
    }

    pub fn write_alias(&mut self, index: usize, result: &MatchResult) {
        let cmd = self
            .fmt_string
            .replace("{file}", &result.file.display().to_string())
            .replace(
                "{line}",
                &result.line.map(|n| n.to_string()).unwrap_or_default(),
            )
            .replace(
                "{column}",
                &result.column.map(|n| n.to_string()).unwrap_or_default(),
            );

        self.buf
            .push_str(&format!("alias {}{index}='{cmd}'\n", self.prefix));
    }

    pub fn flush_to_file(&self, path: &Path) -> std::io::Result<()> {
        fs::write(path, &self.buf)
    }
}
