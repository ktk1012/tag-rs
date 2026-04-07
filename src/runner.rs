use std::io;
use std::process::{Child, Command, ExitStatus, Stdio};

pub fn spawn_piped(prog: &str, args: &[String]) -> io::Result<Child> {
    Command::new(prog)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
}

pub fn passthrough(prog: &str, args: &[String]) -> io::Result<ExitStatus> {
    Command::new(prog)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

pub fn exit_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or_else(|| {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            status.signal().map(|s| 128 + s).unwrap_or(1)
        }
        #[cfg(not(unix))]
        {
            1
        }
    })
}
