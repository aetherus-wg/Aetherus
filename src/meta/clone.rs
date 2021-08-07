//! Clone macro.

/// Succinctly create an access-by-clone method for the given variable.
#[macro_export]
macro_rules! clone {
    ($field:ident, $type:ty) => {
        #[inline]
        #[must_use]
        pub const fn $field(&self) -> $type {
            self.$field
        }
    };

    ($field:ident, $setter:ident, $type:ty) => {
        #[inline]
        #[must_use]
        pub const fn $field(&self) -> $type {
            self.$field
        }

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
        /// Immutable parameter.
        a: i32,
        /// Mutable parameter.
        b: i32,
    }

    impl Testy {
        clone!(a, i32);
        clone!(b, b_mut, i32);
    }


    #[test]
    fn test_clone() {
        let testy = Testy{a: 1, b: 2};

        assert_eq!(testy.a(), 1);
        assert_eq!(testy.b(), 2);
    }

    #[test]
    fn test_mut_clone() {
        let mut testy = Testy{a: 1, b: 2};

        *testy.b_mut() *= -1;

        assert_eq!(testy.b(), -2);
    }
}
