mod log_macros;

use clap::Parser;
use core::fmt::Arguments;
use std::{
    error::Error,
    io::{self, Read, Write},
};
use termios::{tcsetattr, Termios, ECHO, ICANON, ISIG, TCSANOW};

pub trait MonzillaLog {
    fn output(self: &Self, args: Arguments);
    fn warning(self: &Self, args: Arguments);
    fn error(self: &Self, args: Arguments);
}

pub struct MonzillaTool<'a> {
    log: &'a dyn MonzillaLog,
}

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// The JSON5 input file
    #[arg(value_name = "GLOB")]
    input_glob: String,
}
impl<'a> MonzillaTool<'a> {
    pub fn new(log: &'a dyn MonzillaLog) -> MonzillaTool {
        MonzillaTool { log }
    }

    pub fn run(
        self: &mut Self,
        args: impl IntoIterator<Item = std::ffi::OsString>,
    ) -> Result<(), Box<dyn Error>> {
        let _cli = match Cli::try_parse_from(args) {
            Ok(m) => m,
            Err(err) => {
                output!(self.log, "{}", err.to_string());
                return Ok(());
            }
        };

        let stdin = 0i32;
        let termios = Termios::from_fd(stdin)?;
        let mut new_termios = termios.clone();

        // Set new terminal flags to disable canonical mode, echo and signal handling
        new_termios.c_lflag &= !(ICANON | ECHO | ISIG);
        tcsetattr(stdin, TCSANOW, &mut new_termios)?;

        self.run_loop()?;

        tcsetattr(stdin, TCSANOW, &termios)?;
        Ok(())
    }

    fn run_loop(&self) -> Result<(), Box<dyn Error>> {
        let stdout = io::stdout();
        let mut reader = io::stdin();
        let mut buffer = [0; 1]; // read exactly one byte

        println!("Press ^C to stop monitoring and terminate the process. Press ^R to restart the process.");
        stdout.lock().flush()?;

        loop {
            reader.read_exact(&mut buffer).unwrap();

            match buffer[0] {
                3 => {
                    println!("Terminating...");
                    break;
                }
                18 => {
                    println!("Restarting");
                    stdout.lock().flush()?;
                }
                _ => continue,
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        struct TestLogger;

        impl TestLogger {
            fn new() -> TestLogger {
                TestLogger {}
            }
        }

        impl MonzillaLog for TestLogger {
            fn output(self: &Self, _args: Arguments) {}
            fn warning(self: &Self, _args: Arguments) {}
            fn error(self: &Self, _args: Arguments) {}
        }

        let logger = TestLogger::new();
        let mut tool = MonzillaTool::new(&logger);
        let args: Vec<std::ffi::OsString> = vec!["".into(), "--help".into()];

        tool.run(args).unwrap();
    }
}
