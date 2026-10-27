//! Number formatting helpers shared by the exporters.
//!
//! C++ `operator<<(std::ostream&, double)` prints with `%g` semantics and a
//! default precision of 6 significant digits. The native exporters rely on
//! this formatting, so the Rust writers reproduce it exactly.

/// Format a float like C++ `operator<<(std::ostream&, double)` (`%g`, 6
/// significant digits, trailing zeros removed).
pub fn format_cpp_float(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-inf".to_string()
        } else {
            "inf".to_string()
        };
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0".to_string()
        } else {
            "0".to_string()
        };
    }

    const PRECISION: i32 = 6;
    let scientific = format!("{:.*e}", (PRECISION - 1) as usize, value);
    let (mantissa, exponent) = scientific.split_once('e').expect("scientific format");
    let exponent: i32 = exponent.parse().expect("exponent");

    if !(-4..PRECISION).contains(&exponent) {
        let mantissa = strip_trailing_zeros(mantissa);
        let sign = if exponent < 0 { '-' } else { '+' };
        format!("{mantissa}e{sign}{:02}", exponent.abs())
    } else {
        let decimals = (PRECISION - 1 - exponent).max(0) as usize;
        strip_trailing_zeros(&format!("{value:.decimals$}")).to_string()
    }
}

fn strip_trailing_zeros(text: &str) -> &str {
    if !text.contains('.') {
        return text;
    }
    let text = text.trim_end_matches('0');
    text.trim_end_matches('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_cpp_default_stream_formatting() {
        assert_eq!(format_cpp_float(0.0508), "0.0508");
        assert_eq!(format_cpp_float(0.00151638), "0.00151638");
        assert_eq!(format_cpp_float(0.00001778), "1.778e-05");
        assert_eq!(format_cpp_float(4.8), "4.8");
        assert_eq!(format_cpp_float(0.02), "0.02");
        assert_eq!(format_cpp_float(100.0), "100");
        assert_eq!(format_cpp_float(0.0), "0");
        assert_eq!(format_cpp_float(1e-7), "1e-07");
        assert_eq!(format_cpp_float(-0.001), "-0.001");
        assert_eq!(format_cpp_float(1234567.0), "1.23457e+06");
        assert_eq!(format_cpp_float(1.724e-8), "1.724e-08");
        assert_eq!(format_cpp_float(std::f64::consts::PI), "3.14159");
    }
}
