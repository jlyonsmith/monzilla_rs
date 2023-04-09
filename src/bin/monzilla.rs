use colored::Colorize;
use core::fmt::Arguments;
use monzilla_rs::{error, MonzillaLog, MonzillaTool};

struct MonzillaLogger;

impl MonzillaLogger {
    fn new() -> MonzillaLogger {
        MonzillaLogger {}
    }
}

impl MonzillaLog for MonzillaLogger {
    fn output(self: &Self, args: Arguments) {
        println!("{}", args);
    }
    fn warning(self: &Self, args: Arguments) {
        eprintln!("{}", format!("warning: {}", args).yellow());
    }
    fn error(self: &Self, args: Arguments) {
        eprintln!("{}", format!("error: {}", args).red());
    }
}

fn main() {
    let logger = MonzillaLogger::new();

    if let Err(error) = MonzillaTool::new(&logger).run(std::env::args_os()) {
        error!(logger, "{}", error);
        std::process::exit(1);
    }
}
