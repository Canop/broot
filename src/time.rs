/// print the time that executing $timed took
/// but only when the log level is "debug".
/// The goal of this macro is to avoid doing useless
/// `Instant::now` in non Debug executions.
///
/// Examples:
/// ```
/// let sum = time!(Debug, "summing", 2 + 2);
/// let mult = time!(Info, 3 * 4);
/// ```
macro_rules! time {
    ($level: ident, $name: expr, $timed: expr $(,)?) => {{
        use log::Level::*;
        if log_enabled!($level) {
            let start = std::time::Instant::now();
            let value = $timed;
            log!($level, "{} took {:?}", $name, start.elapsed());
            value
        } else {
            $timed
        }
    }};
    ($level: ident, $cat: expr, $name :expr,  $timed: expr $(,)?) => {{
        use log::Level::*;
        if log_enabled!($level) {
            let start = std::time::Instant::now();
            let value = $timed;
            log!($level, "{} on {:?} took {:?}", $cat, $name, start.elapsed());
            value
        } else {
            $timed
        }
    }};
}
