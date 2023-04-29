mod log_macros;

use clap::Parser;
use command_group::CommandGroup;
use core::fmt::Arguments;
use crossbeam::channel::{self, Select, Sender};
use easy_error::ResultExt;
use glob::glob;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::{
    io::{self, Read},
    path::PathBuf,
    process::Command,
    thread,
    time::Duration,
    vec,
};
use termios::{tcsetattr, Termios, ECHO, ICANON, ISIG, TCSANOW};

pub trait MonzillaLog {
    fn disable_color(&self);
    fn plain(&self, args: Arguments);
    fn info(&self, args: Arguments);
    fn warning(&self, args: Arguments);
    fn error(&self, args: Arguments);
}

pub struct MonzillaTool<'a> {
    log: &'a dyn MonzillaLog,
}

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Disable colors in output
    #[arg(long = "no-color", short = 'n', env = "NO_CLI_COLOR")]
    no_color: bool,

    /// Alternative directory to run command in
    #[arg(long = "dir", short = 'd', default_value = ".")]
    cmd_dir: PathBuf,

    /// Debounce time
    #[arg(long = "debounce", short = 't', default_value = "500")]
    debounce_millis: u64,

    /// One or more file globs to watch
    #[arg(long = "glob", short = 'g', num_args = 1.., value_name = "GLOB", required = true)]
    globs: Vec<String>,

    /// The command to run
    #[arg(value_name = "COMMAND", trailing_var_arg = true, required = true)]
    command: Vec<String>,
}

enum UiEvent {
    CtrlC,
    CtrlR,
}

impl<'a> MonzillaTool<'a> {
    pub fn new(log: &'a dyn MonzillaLog) -> MonzillaTool {
        MonzillaTool { log }
    }

    pub fn run(
        self: &mut Self,
        args: impl IntoIterator<Item = std::ffi::OsString>,
    ) -> Result<(), easy_error::Error> {
        let cli = match Cli::try_parse_from(args) {
            Ok(m) => m,
            Err(err) => {
                plain!(self.log, "{}", err.to_string());
                return Ok(());
            }
        };

        if cli.no_color {
            self.log.disable_color();
        }

        let stdin = 0i32;
        let termios = Termios::from_fd(stdin).context("Cannot read terminal I/O configuration")?;
        let mut new_termios = termios.clone();

        // Set new terminal flags to disable canonical mode, echo and signal handling
        new_termios.c_lflag &= !(ICANON | ECHO | ISIG);
        tcsetattr(stdin, TCSANOW, &mut new_termios)
            .context("Cannot set terminal I/O configuration")?;

        let result = self.inner_run(&cli);

        match tcsetattr(stdin, TCSANOW, &termios) {
            Ok(_) => (),
            Err(err) => warning!(
                self.log,
                "Unable to reset terminal I/O: {}",
                err.to_string()
            ),
        }

        result
    }

    fn inner_run(&self, cli: &Cli) -> Result<(), easy_error::Error> {
        // Get the list of files & directories to watch
        let mut paths: Vec<PathBuf> = vec![];

        for glob_str in cli.globs.iter() {
            for entry in glob(glob_str).context("Failed to read glob pattern")? {
                match entry {
                    Ok(path) => paths.push(path),
                    Err(e) => warning!(self.log, "'{}': {}", &glob_str, e.to_string()),
                }
            }
        }

        let (ui_tx, ui_rx) = channel::unbounded::<UiEvent>();
        let (notify_tx, notify_rx) = channel::unbounded::<DebounceEventResult>();
        let mut debouncer =
            new_debouncer(Duration::from_millis(cli.debounce_millis), None, notify_tx)
                .context("Unable to create watcher")?;

        // Add all paths to be watched
        for path in paths {
            debouncer
                .watcher()
                .watch(&path, RecursiveMode::NonRecursive)
                .context(format!("Unable to watch file '{}'", path.to_string_lossy()))?;
        }
        thread::spawn(move || MonzillaTool::ui_thread_proc(ui_tx));

        let command = cli.command.join(" ").to_string();
        let mut exiting = false;

        // Create the Select object to wait on
        let mut sel = Select::new();
        let notify_index = sel.recv(&notify_rx);
        let ui_index = sel.recv(&ui_rx);

        loop {
            info!(self.log, "Running command '{}'", &command);
            info!(self.log, "Control+C to exit/Control+R to restart");

            let mut child = Command::new(&cli.command[0])
                .current_dir(&cli.cmd_dir)
                .args(&cli.command[1..])
                .group_spawn()
                .context(format!("Could not start {}", &command))?;

            loop {
                let operation = sel.select();

                match operation.index() {
                    i if i == notify_index => {
                        let _ = operation.recv(&notify_rx);
                        break;
                    }
                    i if i == ui_index => {
                        let ui_event = operation.recv(&ui_rx).unwrap();

                        match ui_event {
                            UiEvent::CtrlC => {
                                info!(self.log, "Terminating...");
                                exiting = true;
                                break;
                            }
                            UiEvent::CtrlR => {
                                info!(self.log, "Restarting...");
                                break;
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }

            let exit_status = child
                .try_wait()
                .context("Cannot get child process context")?;

            if let None = exit_status {
                warning!(self.log, "Killing process group {}", child.id());

                child
                    .kill()
                    .context(format!("Could not kill process group {}", child.id()))?;
            }

            child.wait().context("Could not wait for process to exit")?;

            if exiting {
                break;
            }
        }

        Ok(())
    }

    fn ui_thread_proc(tx: Sender<UiEvent>) {
        let mut reader = io::stdin();
        let mut buffer = [0; 1]; // read exactly one byte

        loop {
            reader.read_exact(&mut buffer).unwrap();

            match buffer[0] {
                3 => {
                    tx.send(UiEvent::CtrlC).unwrap();
                }
                18 => {
                    tx.send(UiEvent::CtrlR).unwrap();
                }
                _ => continue,
            }
        }
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
            fn disable_color(&self) {}
            fn plain(self: &Self, _args: Arguments) {}
            fn info(self: &Self, _args: Arguments) {}
            fn warning(self: &Self, _args: Arguments) {}
            fn error(self: &Self, _args: Arguments) {}
        }

        let logger = TestLogger::new();
        let mut tool = MonzillaTool::new(&logger);
        let args: Vec<std::ffi::OsString> = vec!["".into(), "--help".into()];

        tool.run(args).unwrap();
    }
}
