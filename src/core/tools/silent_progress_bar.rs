//! Silent progress-Bar implementation.

/// Silent progress-bar structure.
pub struct SilentProgressBar {
    /// Current value.
    count: usize,
    /// Total target value.
    total: usize,
}

impl SilentProgressBar {
    /// Construct a new instance.
    #[inline]
    #[must_use]
    pub fn new(total: usize) -> Self {
        debug_assert!(total > 0);

        Self { count: 0, total }
    }

    /// Request a block of values to work on.
    /// Return the requested block if available.
    /// If there is not enough, return the remaining block.
    /// If there are none at all, return None.
    #[inline]
    pub fn block(&mut self, size: usize) -> Option<(usize, usize)> {
        debug_assert!(size > 0);

        if self.count >= self.total {
            None
        } else {
            let remaining = self.total - self.count;
            let alloc = size.min(remaining);

            let start = self.count;
            let end = start + alloc;

            self.count += alloc;

            Some((start, end))
        }
    }

    /// Check if the progress bar is complete.
    #[inline]
    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.count >= self.total
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_new_silent_progress_bar() {
        use super::SilentProgressBar;

        let total = 10;
        let _ = SilentProgressBar::new(total);
    }

    #[test]
    #[should_panic]
    fn test_new_silent_progress_bar_fail() {
        use super::SilentProgressBar;

        let total = 0;
        let _ = SilentProgressBar::new(total);
    }

    #[test]
    fn test_block() {
        use super::SilentProgressBar;

        let total = 10;
        let mut pb = SilentProgressBar::new(total);

        let size = 3;
        let (start, end) = pb.block(size).expect("Failed to get block.");

        assert_eq!(start, 0);
        assert_eq!(end, size);
    }

    #[test]
    fn test_block_remaining() {
        use super::SilentProgressBar;

        let total = 10;
        let mut pb = SilentProgressBar::new(total);

        let size = 3;
        let (start, end) = pb.block(size).expect("Failed to get block.");

        assert_eq!(start, 0);
        assert_eq!(end, size);

        let size = 3;
        let (start, end) = pb.block(size).expect("Failed to get block.");

        assert_eq!(start, 3);
        assert_eq!(end, 6);

        let size = 3;
        let (start, end) = pb.block(size).expect("Failed to get block.");

        assert_eq!(start, 6);
        assert_eq!(end, 9);

        let size = 3;
        let (start, end) = pb.block(size).expect("Failed to get block.");

        assert_eq!(start, 9);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_block_none() {
        use super::SilentProgressBar;

        let total = 10;
        let mut pb = SilentProgressBar::new(total);

        let size = 3;
        let (start, end) = pb.block(size).expect("Failed to get block.");

        assert_eq!(start, 0);
        assert_eq!(end, size);

        let size = 3;
        let (start, end) = pb.block(size).expect("Failed to get block.");

        assert_eq!(start, 3);
        assert_eq!(end, 6);

        let size = 3;
        let (start, end) = pb.block(size).expect("Failed to get block.");

        assert_eq!(start, 6);
        assert_eq!(end, 9);

        let size = 3;
        let (start, end) = pb.block(size).expect("Failed to get block.");

        assert_eq!(start, 9);
        assert_eq!(end, 10);

        let size = 3;
        let block = pb.block(size);

        assert!(block.is_none());
    }

    #[test]
    fn test_is_done() {
        use super::SilentProgressBar;

        let total = 10;
        let mut pb = SilentProgressBar::new(total);

        assert!(!pb.is_done());

        let size = 3;
        let _ = pb.block(size).expect("Failed to get block.");

        assert!(!pb.is_done());

        let size = 3;
        let _ = pb.block(size).expect("Failed to get block.");

        assert!(!pb.is_done());

        let size = 3;
        let _ = pb.block(size).expect("Failed to get block.");

        assert!(!pb.is_done());

        let size = 3;
        let _ = pb.block(size).expect("Failed to get block.");

        assert!(pb.is_done());
    }
}