use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::mode::MatchResult;

/// Escape a string for use inside single quotes in shell.
/// Replaces `'` with `'\''` (end quote, escaped quote, start quote).
fn shell_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

pub struct AliasWriter {
    prefix: String,
    fmt_string: String,
    buf: String,
    count: usize,
}

impl AliasWriter {
    pub fn new(prefix: String, fmt_string: String) -> Self {
        Self {
            prefix,
            fmt_string,
            buf: String::new(),
            count: 0,
        }
    }

    pub fn write_alias(&mut self, index: usize, result: &MatchResult) {
        let file_escaped = shell_escape(&result.file.display().to_string());
        let cmd = self
            .fmt_string
            .replace("{file}", &file_escaped)
            .replace(
                "{line}",
                &result.line.map(|n| n.to_string()).unwrap_or_default(),
            )
            .replace(
                "{column}",
                &result.column.map(|n| n.to_string()).unwrap_or_default(),
            );

        let _ = writeln!(self.buf, "alias {}{index}='{cmd}'", self.prefix);
        let _ = writeln!(self.buf, "export {}{index}='{file_escaped}'", self.prefix,);
        self.count = index;
    }

    pub fn write_var(&mut self, index: usize, value: &str) {
        let escaped = shell_escape(value);
        let _ = writeln!(self.buf, "export {}{index}='{escaped}'", self.prefix);
        self.count = index;
    }

    pub fn flush_to_file(&self, path: &Path) -> std::io::Result<()> {
        let mut out = String::new();
        // Clear previous vars/aliases to prevent stale references.
        // Unset a range beyond current count to cover leftovers from prior runs.
        let clear_up_to = self.count + 50;
        let _ = write!(out, "unset");
        for i in 1..=clear_up_to {
            let _ = write!(out, " {}{i}", self.prefix);
        }
        let _ = writeln!(out);
        out.push_str(&self.buf);
        fs::write(path, &out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn write_alias_emits_alias_and_export() {
        let mut w = AliasWriter::new("e".into(), r#"vim "{file}""#.into());
        let result = MatchResult {
            file: PathBuf::from("/src/main.rs"),
            line: None,
            column: None,
        };
        w.write_alias(1, &result);

        assert!(w.buf.contains("alias e1='vim \"/src/main.rs\"'"));
        assert!(w.buf.contains("export e1='/src/main.rs'"));
    }

    #[test]
    fn write_alias_with_line_column() {
        let mut w = AliasWriter::new(
            "e".into(),
            r#"vim -c "call cursor({line}, {column})" "{file}""#.into(),
        );
        let result = MatchResult {
            file: PathBuf::from("/src/main.rs"),
            line: Some(10),
            column: Some(5),
        };
        w.write_alias(1, &result);

        assert!(w.buf.contains("call cursor(10, 5)"));
        assert!(w.buf.contains("export e1='/src/main.rs'"));
    }

    #[test]
    fn write_var_emits_export() {
        let mut w = AliasWriter::new("e".into(), String::new());
        w.write_var(3, "main");

        assert!(w.buf.contains("export e3='main'"));
    }

    #[test]
    fn flush_prepends_unset() {
        let mut w = AliasWriter::new("e".into(), r#"vim "{file}""#.into());
        let result = MatchResult {
            file: PathBuf::from("/a.rs"),
            line: None,
            column: None,
        };
        w.write_alias(1, &result);
        w.write_alias(
            2,
            &MatchResult {
                file: PathBuf::from("/b.rs"),
                line: None,
                column: None,
            },
        );

        let dir = std::env::temp_dir();
        let path = dir.join("tag_test_flush");
        w.flush_to_file(&path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        // unset line should come before alias lines
        let unset_pos = content.find("unset").unwrap();
        let alias_pos = content.find("alias").unwrap();
        assert!(unset_pos < alias_pos);
        // should unset beyond the count
        assert!(content.contains("e52"));

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn custom_prefix() {
        let mut w = AliasWriter::new("t".into(), r#"vim "{file}""#.into());
        let result = MatchResult {
            file: PathBuf::from("/x.rs"),
            line: None,
            column: None,
        };
        w.write_alias(1, &result);

        assert!(w.buf.contains("alias t1="));
        assert!(w.buf.contains("export t1="));
    }
}
