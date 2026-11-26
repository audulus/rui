//! rui under gui-bench: one window per run that shows the scenario the command line names.

#![forbid(unsafe_code)]

#[path = "spec/cli.rs"]
mod cli;
#[path = "spec/dashboard.rs"]
mod dash;
mod dashboard;

use rui::{WindowConfig, any_view, rui_with_config};

use crate::cli::Scenario;

const NAME: &str = "rui-bench";

fn main() {
    let scenario = cli::from_env(NAME);
    let config = WindowConfig {
        title: scenario.title().to_owned(),
        size: Some([dash::WINDOW.0, dash::WINDOW.1]),
        decorations: false,
    };
    let content = match scenario {
        Scenario::App { count } => any_view(dashboard::view(count)),
        Scenario::List { count } => any_view(dashboard::list::view(count)),
    };
    rui_with_config(content, config);
}
