//! Access macro.

/// Create an access-by-reference method for the given variable.
#[macro_export]
macro_rules! access {
    ($field:ident: $type:ty) => {
        #[inline]
        #[must_use]
        pub const fn $field(&self) -> &$type {
            &self.$field
        }
    };

    ($field:ident, $setter:ident: $type:ty) => {
        #[inline]
        #[must_use]
        pub const fn $field(&self) -> &$type {
            &self.$field
        }

        #[allow(clippy::mut_mut)]
        #[inline]
        #[must_use]
        pub fn $setter(&mut self) -> &mut $type {
            &mut self.$field
        }
    };
}

#[cfg(test)]
mod tests {
    /// Test implementation structure.
    pub struct Testy {
        /// Mutable parameter.
        a: String,
        /// Immutable parameter.
        b: String,
    }

    impl Testy {
        access!(a: String);
        access!(b, b_mut: String);
    }

    #[test]
    fn test_access() {
        let testy = Testy {
            a: "one".to_owned(),
            b: "two".to_owned(),
        };

        assert_eq!(testy.a(), &"one");
        assert_eq!(testy.b(), &"two");
    }

    #[test]
    fn test_mut_access() {
        let mut testy = Testy {
            a: "one".to_owned(),
            b: "two".to_owned(),
        };

        *testy.b_mut() = "three".to_owned();

        assert_eq!(testy.b(), &"three");
    }
}
