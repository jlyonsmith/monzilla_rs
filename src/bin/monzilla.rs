use core::fmt::Arguments;
use monzilla_rs::{error, MonzillaLog, MonzillaTool};
use yansi::Paint;

struct MonzillaLogger;

impl MonzillaLogger {
    fn new() -> MonzillaLogger {
        MonzillaLogger {}
    }
}

impl MonzillaLog for MonzillaLogger {
    fn disable_color(&self) {
        Paint::disable();
    }

    fn plain(&self, args: Arguments) {
        eprintln!("{}", args);
    }
    fn info(&self, args: Arguments) {
        eprintln!("{}", Paint::green(format!("[monzilla] {}", args)));
    }
    fn warning(&self, args: Arguments) {
        eprintln!("{}", Paint::yellow(format!("[monzilla] warning: {}", args)));
    }
    fn error(&self, args: Arguments) {
        eprintln!("{}", Paint::red(format!("[monzilla] error: {}", args)));
    }
}

fn main() {
    let logger = MonzillaLogger::new();

    if let Err(error) = MonzillaTool::new(&logger).run(std::env::args_os()) {
        error!(logger, "{}", error);
        std::process::exit(1);
    }
}
