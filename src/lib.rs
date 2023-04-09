mod log_macros;

use clap::Parser;
use core::fmt::Arguments;
use std::error::Error;

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
