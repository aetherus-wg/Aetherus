//! Greek alphabet.

/// Greek letters.
pub enum Greek {
    /// First letter.
    Alpha,
    /// Second letter.
    Beta,
    /// Third letter.
    Gamma,
}

/// Greek Alpha convenience indexing constant.
pub const ALPHA: usize = Greek::Alpha as usize;

/// Greek Beta convenience indexing constant.
pub const BETA: usize = Greek::Beta as usize;

/// Greek Gamma convenience indexing constant.
pub const GAMMA: usize = Greek::Gamma as usize;

#[cfg(test)]
mod tests {
    use super::{ALPHA, BETA, GAMMA};

    /// Checking that each index pulls back the expected index.
    #[test]
    fn greek_list_test() {
        let items = vec![1.0, 2.0, 3.0];
        assert_eq!(items[ALPHA], 1.0);
        assert_eq!(items[BETA], 2.0);
        assert_eq!(items[GAMMA], 3.0);
    }
}
