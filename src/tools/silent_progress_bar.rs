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
