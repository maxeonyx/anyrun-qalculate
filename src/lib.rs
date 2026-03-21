use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use serde::Deserialize;
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex, OnceLock};

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
    calculator: Arc<SharedCalculator>,
    config: Config,
}

static DEFAULT_CALCULATOR: OnceLock<Result<Arc<SharedCalculator>, CalculationError>> =
    OnceLock::new();
#[cfg(test)]
static TEST_PLUGIN_INIT: OnceLock<()> = OnceLock::new();

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

#[derive(Clone, Debug, PartialEq, Eq)]
enum CalculationError {
    ExpressionContainsNul,
    NativeHandleInitFailed,
    NativeReturnedInvalidUtf8,
    NativeReturnedNull,
}

impl Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpressionContainsNul => f.write_str("expression contains an interior NUL byte"),
            Self::NativeHandleInitFailed => f.write_str("failed to initialize libqalculate handle"),
            Self::NativeReturnedInvalidUtf8 => f.write_str("native layer returned invalid UTF-8"),
            Self::NativeReturnedNull => f.write_str("native layer returned a null pointer"),
        }
    }
}

struct NativeCalculationResult(*mut c_char);
struct NativeCalculator(*mut std::ffi::c_void);

struct SharedCalculator(Mutex<NativeCalculator>);

unsafe impl Send for NativeCalculator {}

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
            qalculate_free_string(self.0);
        }
    }
}

impl NativeCalculator {
    fn new() -> Result<Self, CalculationError> {
        let handle = unsafe { qalculate_new() };

        if handle.is_null() {
            return Err(CalculationError::NativeHandleInitFailed);
        }

        Ok(Self(handle))
    }

    fn calculate(&self, expression: &CString) -> Result<String, CalculationError> {
        let result = NativeCalculationResult::from_raw(unsafe {
            qalculate_calculate(self.0, expression.as_ptr())
        })?;

        result.to_owned_string()
    }
}

impl Drop for NativeCalculator {
    fn drop(&mut self) {
        unsafe {
            qalculate_free(self.0);
        }
    }
}

impl SharedCalculator {
    fn new() -> Result<Arc<Self>, CalculationError> {
        Ok(Arc::new(Self(Mutex::new(NativeCalculator::new()?))))
    }

    fn calculate(&self, expression: &CString) -> Result<String, CalculationError> {
        let calculator = self.0.lock().expect("qalculate mutex poisoned");
        calculator.calculate(expression)
    }
}

unsafe extern "C" {
    fn qalculate_new() -> *mut std::ffi::c_void;
    fn qalculate_free(handle: *mut std::ffi::c_void);
    fn qalculate_calculate(handle: *mut std::ffi::c_void, expression: *const c_char)
        -> *mut c_char;
    fn qalculate_free_string(value: *mut c_char);
}

#[init]
fn init(config_dir: RString) -> State {
    State {
        calculator: default_calculator().expect("libqalculate should initialize"),
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

    match calculate_expression_with(&state.calculator, expression) {
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

    if !input_has_calculation_signal(input) {
        return None;
    }

    Some(input)
}

fn input_has_calculation_signal(input: &str) -> bool {
    input.chars().any(|ch| {
        ch.is_ascii_digit() || matches!(ch, '+' | '-' | '*' | '/' | '%' | '^' | '=' | '(' | ')')
    })
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
    ron::from_str(contents).map_err(|source| ConfigLoadError::Parse {
        path: config_path.to_string(),
        source,
    })
}

fn default_calculator() -> Result<Arc<SharedCalculator>, CalculationError> {
    DEFAULT_CALCULATOR
        .get_or_init(SharedCalculator::new)
        .as_ref()
        .map(Arc::clone)
        .map_err(Clone::clone)
}

#[cfg(test)]
fn calculate_expression(expression: &str) -> Result<String, CalculationError> {
    let calculator = default_calculator()?;

    calculate_expression_with(&calculator, expression)
}

fn calculate_expression_with(
    calculator: &SharedCalculator,
    expression: &str,
) -> Result<String, CalculationError> {
    let expression = normalize_expression(expression);
    let expression =
        CString::new(expression.as_ref()).map_err(|_| CalculationError::ExpressionContainsNul)?;

    calculator.calculate(&expression)
}

fn normalize_expression(expression: &str) -> Cow<'_, str> {
    if expression.contains("% of ") {
        return Cow::Owned(expression.replace("% of ", "%*"));
    }

    if expression.contains(" in ") {
        return Cow::Owned(expression.replace(" in ", " to "));
    }

    Cow::Borrowed(expression)
}

#[cfg(test)]
mod tests {
    use super::{
        anyrun_internal_get_matches, anyrun_internal_init, calculate_expression,
        default_calculator, parse_config, CalculationError, ConfigLoadError, TEST_PLUGIN_INIT,
    };
    use std::time::{Duration, Instant};

    fn ensure_plugin_initialized() {
        TEST_PLUGIN_INIT.get_or_init(|| {
            let _ = default_calculator();
            anyrun_internal_init("/tmp/anyrun-qalculate-tests".into());

            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(2) {
                if !anyrun_internal_get_matches("1 + 1".into()).is_empty() {
                    return;
                }

                std::thread::sleep(Duration::from_millis(10));
            }

            panic!("timed out waiting for plugin init to finish");
        });
    }

    fn plugin_matches(input: &str) -> Vec<String> {
        ensure_plugin_initialized();

        anyrun_internal_get_matches(input.into())
            .into_iter()
            .map(|matched| matched.title.to_string())
            .collect()
    }

    #[test]
    fn ffi_stub_returns_placeholder_text() {
        assert_eq!(calculate_expression("1 + 1").as_deref(), Ok("2"));
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

    #[test]
    fn plugin_returns_exact_basic_arithmetic_result_within_latency_budget() {
        let _ = plugin_matches("1 + 1");

        let start = Instant::now();
        let matches = plugin_matches("2 + 2");
        let elapsed = start.elapsed();

        assert_eq!(matches.len(), 1, "expected one arithmetic result");
        assert_eq!(matches[0], "4");
        assert!(
            elapsed < Duration::from_millis(50),
            "expected a hot calculation in under 50ms, got {:?}",
            elapsed
        );
    }

    #[test]
    fn plugin_formats_fractional_division_as_a_decimal() {
        let titles = plugin_matches("10 / 3");

        assert_eq!(titles.len(), 1, "expected one division result");
        assert!(
            titles[0].contains('.'),
            "expected decimal output, got {:?}",
            titles[0]
        );
        assert!(
            titles[0].starts_with("3."),
            "expected 3.x output, got {:?}",
            titles[0]
        );
    }

    #[test]
    fn plugin_converts_units_to_pounds() {
        let titles = plugin_matches("5 kg to lbs");

        assert_eq!(titles.len(), 1, "expected one unit-conversion result");
        assert!(
            titles[0].contains("11"),
            "expected pounds magnitude in result, got {:?}",
            titles[0]
        );
        assert!(
            titles[0].contains("lb"),
            "expected pound unit in result, got {:?}",
            titles[0]
        );
    }

    #[test]
    fn plugin_returns_a_currency_conversion_result() {
        let titles = plugin_matches("1 usd in nzd");

        assert_eq!(titles.len(), 1, "expected one currency-conversion result");
        assert!(
            !titles[0].trim().is_empty(),
            "expected non-empty currency output"
        );
        assert!(
            titles[0].contains("NZD"),
            "expected NZD in result, got {:?}",
            titles[0]
        );
        assert!(
            !titles[0].contains("USD in NZD"),
            "expected computed output rather than echoed expression, got {:?}",
            titles[0]
        );
    }

    #[test]
    fn plugin_handles_natural_language_percentages() {
        let titles = plugin_matches("20% of 500");

        assert_eq!(titles.len(), 1, "expected one percentage result");
        assert_eq!(titles[0], "100");
    }

    #[test]
    fn plugin_filters_out_empty_and_garbage_input() {
        assert!(
            plugin_matches("").is_empty(),
            "expected no result for empty input"
        );
        assert!(
            plugin_matches("asdfghjkl").is_empty(),
            "expected no result for garbage input"
        );
    }
}
