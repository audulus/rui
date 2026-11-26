// Vendored from gui-bench (https://github.com/zortax/gui-bench), apps/shared/cli.rs,
// as of audulus/gui-bench@329923a. Copyright the gui-bench authors; licensed under the Apache
// License 2.0 (see examples/gui_bench/LICENSE-APACHE). Keep it in sync with gui-bench, so the
// app draws what the benchmark's probes expect.

//! The command line of every app: `--scenario NAME` picks the scenario (`app` by default) and
//! `--rows N` its size (100 by default).

#![allow(dead_code)]

use std::fmt;

/// The row count without `--rows`.
const DEFAULT_ROWS: usize = 100;

/// The usage line of every app.
pub const USAGE: &str = "[--scenario app|list] [--rows N]";

/// A scenario and its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    /// The workbench app of `dashboard.rs` with `count` orders and `count` messages, all of them
    /// real elements.
    App {
        /// The number of orders and of messages.
        count: usize,
    },
    /// The order rows of `dashboard.rs` in the framework's virtual list: `count` rows of data,
    /// of which only the visible ones exist.
    List {
        /// The number of rows of data.
        count: usize,
    },
}

impl Scenario {
    /// Returns the window title of the scenario.
    pub fn title(&self) -> &'static str {
        match self {
            Self::App { .. } => "gui-bench app",
            Self::List { .. } => "gui-bench list",
        }
    }
}

/// A command line that names no valid scenario.
#[derive(Debug, PartialEq, Eq)]
pub enum CliError {
    /// An option is the last argument and has no value.
    MissingValue(&'static str),
    /// The value of a numeric option is no whole number.
    BadNumber(&'static str, String),
    /// A scenario name the program does not know.
    UnknownScenario(String),
    /// An argument the program does not know.
    Unknown(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingValue(option) => write!(f, "{option} needs a value"),
            Self::BadNumber(option, value) => {
                write!(f, "{option} takes a whole number, got `{value}`")
            }
            Self::UnknownScenario(name) => write!(f, "unknown scenario `{name}`"),
            Self::Unknown(argument) => write!(f, "unknown argument `{argument}`"),
        }
    }
}

/// Reads the scenario from the arguments after the program name.
pub fn parse(mut args: impl Iterator<Item = String>) -> Result<Scenario, CliError> {
    let mut scenario = String::from("app");
    let mut count = DEFAULT_ROWS;
    while let Some(argument) = args.next() {
        let (option, inline) = match argument.split_once('=') {
            Some((option, value)) => (option.to_owned(), Some(value.to_owned())),
            None => (argument.clone(), None),
        };
        let mut value = |name: &'static str| {
            inline
                .clone()
                .or_else(|| args.next())
                .ok_or(CliError::MissingValue(name))
        };
        match option.as_str() {
            "--scenario" => scenario = value("--scenario")?,
            "--rows" => {
                let text = value("--rows")?;
                count = text
                    .parse()
                    .map_err(|_| CliError::BadNumber("--rows", text))?;
            }
            _ => return Err(CliError::Unknown(argument)),
        }
    }
    match scenario.as_str() {
        "app" => Ok(Scenario::App { count }),
        "list" => Ok(Scenario::List { count }),
        _ => Err(CliError::UnknownScenario(scenario)),
    }
}

/// Parses the process arguments, and exits with status 2 and the usage on error. `name` is the
/// program name in messages.
pub fn from_env(name: &str) -> Scenario {
    match parse(std::env::args().skip(1)) {
        Ok(scenario) => {
            eprintln!("{name}: {scenario:?}");
            scenario
        }
        Err(error) => {
            eprintln!("{name}: {error}");
            eprintln!("usage: {name} {USAGE}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> Result<Scenario, CliError> {
        parse(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn no_arguments_give_the_default_app() {
        assert_eq!(
            run(&[]),
            Ok(Scenario::App {
                count: DEFAULT_ROWS
            })
        );
    }

    #[test]
    fn both_forms_set_the_values() {
        assert_eq!(
            run(&["--scenario", "list", "--rows", "10000"]),
            Ok(Scenario::List { count: 10000 })
        );
        assert_eq!(
            run(&["--scenario=app", "--rows=30"]),
            Ok(Scenario::App { count: 30 })
        );
    }

    #[test]
    fn bad_input_is_an_error() {
        assert_eq!(run(&["--rows"]), Err(CliError::MissingValue("--rows")));
        assert!(matches!(
            run(&["--rows", "x"]),
            Err(CliError::BadNumber("--rows", _))
        ));
        assert!(matches!(
            run(&["--scenario", "menu"]),
            Err(CliError::UnknownScenario(_))
        ));
        assert!(matches!(run(&["-v"]), Err(CliError::Unknown(_))));
    }
}
