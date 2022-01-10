//! Red-Green-Blue colour representation.

/// RGB format.
pub enum Rgb {
    /// Red channel.
    Red,
    /// Green channel.
    Green,
    /// Blue channel.
    Blue,
}

/// Red convenience indexing constant.
pub const RED: usize = Rgb::Red as usize;

/// Green convenience indexing constant.
pub const GREEN: usize = Rgb::Green as usize;

/// Blue convenience indexing constant.
pub const BLUE: usize = Rgb::Blue as usize;

#[cfg(test)]
mod tests {
    use super::{BLUE, GREEN, RED};

    /// Checking that each index pulls back the expected index.
    #[test]
    fn colour_index_test() {
        let rgb_value = vec![255, 128, 0];
        assert_eq!(rgb_value[RED], 255);
        assert_eq!(rgb_value[GREEN], 128);
        assert_eq!(rgb_value[BLUE], 0);
    }
}
