/// Returns the number of digits in a number,
/// in order to compute the char width needed to
/// display it
pub fn number_digits(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    (n as f64).log10().floor() as usize + 1
}

/// Returns a whitespace str such that when printed along with `n`
/// it occupies at least `width` chars.
pub fn whitespace_padding(n: usize, width: usize) -> String {
    let remaining = width.saturating_sub(number_digits(n));
    " ".repeat(remaining)
}
