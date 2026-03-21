use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use serde::Deserialize;
use std::ffi::{CStr, CString};
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::os::raw::c_char;

#[derive(Debug, Deserialize, PartialEq, Eq)]
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

#[derive(Debug)]
enum ConfigLoadError {
    NotFound,
    Read {
        path: String,
        source: io::Error,
    },
    Parse {
        path: String,
        source: ron::error::SpannedError,
    },
}

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => f.write_str("config file not found"),
            Self::Read { path, source } => write!(f, "failed to read config at {path}: {source}"),
            Self::Parse { path, source } => {
                write!(f, "failed to parse config at {path}: {source}")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum CalculationError {
    ExpressionContainsNul,
    NativeReturnedInvalidUtf8,
    NativeReturnedNull,
}

impl Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpressionContainsNul => f.write_str("expression contains an interior NUL byte"),
            Self::NativeReturnedInvalidUtf8 => f.write_str("native layer returned invalid UTF-8"),
            Self::NativeReturnedNull => f.write_str("native layer returned a null pointer"),
        }
    }
}

struct NativeCalculationResult(*mut c_char);

impl NativeCalculationResult {
    fn from_raw(value: *mut c_char) -> Result<Self, CalculationError> {
        if value.is_null() {
            return Err(CalculationError::NativeReturnedNull);
        }

        Ok(Self(value))
    }

    fn to_owned_string(&self) -> Result<String, CalculationError> {
        unsafe { CStr::from_ptr(self.0) }
            .to_str()
            .map(str::to_owned)
            .map_err(|_| CalculationError::NativeReturnedInvalidUtf8)
    }
}

impl Drop for NativeCalculationResult {
    fn drop(&mut self) {
        unsafe {
            qalculate_stub_free_string(self.0);
        }
    }
}

unsafe extern "C" {
    fn qalculate_stub_calculate(expression: *const c_char) -> *mut c_char;
    fn qalculate_stub_free_string(value: *mut c_char);
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
    let Some(expression) = matchable_input(&input, &state.config) else {
        return RVec::new();
    };

    match calculate_expression(expression) {
        Ok(title) => vec![build_match(title)].into(),
        Err(error) => {
            eprintln!("[qalculate] Failed to calculate '{expression}': {error}");
            RVec::new()
        }
    }
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    HandleResult::Copy(selection.title.into_bytes())
}

fn build_match(title: String) -> Match {
    Match {
        title: title.into(),
        description: ROption::RNone,
        use_pango: false,
        icon: ROption::RNone,
        id: ROption::RNone,
    }
}

fn matchable_input<'a>(input: &'a str, config: &Config) -> Option<&'a str> {
    let input = input.strip_prefix(&config.prefix)?.trim();

    if input.is_empty() {
        return None;
    }

    Some(input)
}

fn load_config(config_dir: &str) -> Config {
    match read_config(config_dir) {
        Ok(config) => config,
        Err(ConfigLoadError::NotFound) => Config::default(),
        Err(error) => {
            eprintln!("[qalculate] {error}");
            Config::default()
        }
    }
}

fn read_config(config_dir: &str) -> Result<Config, ConfigLoadError> {
    let config_path = format!("{config_dir}/qalculate.ron");
    let contents = match fs::read_to_string(&config_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(ConfigLoadError::NotFound);
        }
        Err(error) => {
            return Err(ConfigLoadError::Read {
                path: config_path,
                source: error,
            });
        }
    };

    parse_config(&config_path, &contents)
}

fn parse_config(config_path: &str, contents: &str) -> Result<Config, ConfigLoadError> {
    let _ = config_path;

    Ok(ron::from_str(contents).unwrap_or_default())
}

fn calculate_expression(expression: &str) -> Result<String, CalculationError> {
    let expression = CString::new(expression).map_err(|_| CalculationError::NativeReturnedNull)?;
    let result = NativeCalculationResult::from_raw(unsafe {
        qalculate_stub_calculate(expression.as_ptr())
    })?;

    result.to_owned_string()
}

#[cfg(test)]
mod tests {
    use super::{calculate_expression, parse_config, CalculationError, ConfigLoadError};

    #[test]
    fn ffi_stub_returns_placeholder_text() {
        assert_eq!(
            calculate_expression("1 + 1").as_deref(),
            Ok("qalculate stub: 1 + 1")
        );
    }

    #[test]
    fn ffi_rejects_interior_nul_with_explicit_error() {
        assert_eq!(
            calculate_expression("1 +\0 1"),
            Err(CalculationError::ExpressionContainsNul)
        );
    }

    #[test]
    fn parse_config_reports_invalid_ron() {
        assert!(
            matches!(
                parse_config("/tmp/qalculate.ron", "not = valid"),
                Err(ConfigLoadError::Parse { .. })
            ),
            "invalid config should surface a parse error"
        );
    }
}
