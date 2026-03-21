use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct Config {
    prefix: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix: String::new(),
        }
    }
}

struct State {
    config: Config,
}

#[init]
fn init(config_dir: RString) -> State {
    State {
        config: load_config(&config_dir),
    }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Qalculate".into(),
        icon: "accessories-calculator".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &State) -> RVec<Match> {
    let input = if let Some(input) = input.strip_prefix(&state.config.prefix) {
        input.trim()
    } else {
        return RVec::new();
    };

    if input.is_empty() {
        return RVec::new();
    }

    match calculate_expression(input) {
        Some(title) => vec![Match {
            title: title.into(),
            description: ROption::RNone,
            use_pango: false,
            icon: ROption::RNone,
            id: ROption::RNone,
        }]
        .into(),
        None => RVec::new(),
    }
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    HandleResult::Copy(selection.title.into_bytes())
}

fn load_config(config_dir: &str) -> Config {
    let config_path = format!("{config_dir}/qalculate.ron");

    match fs::read_to_string(config_path) {
        Ok(contents) => ron::from_str(&contents).unwrap_or_else(|_| Config::default()),
        Err(_) => Config::default(),
    }
}

fn calculate_expression(_expression: &str) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::calculate_expression;

    #[test]
    fn ffi_stub_returns_placeholder_text() {
        assert_eq!(
            calculate_expression("1 + 1").as_deref(),
            Some("qalculate stub: 1 + 1")
        );
    }
}
