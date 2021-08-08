//! Clone macro.

/// Succinctly create an access-by-clone method for the given variable.
#[macro_export]
macro_rules! clone {
    ($field:ident: $type:ty) => {
        #[inline]
        #[must_use]
        pub const fn $field(&self) -> $type {
            self.$field
        }
    };

    ($field:ident, $setter:ident: $type:ty) => {
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
    use crate::core::Int;

    /// Test implementation structure.
    pub struct Testy {
        /// Immutable parameter.
        a: Int,
        /// Mutable parameter.
        b: Int,
    }

    impl Testy {
        clone!(a: Int);
        clone!(b, b_mut: Int);
    }

    #[test]
    fn test_clone() {
        let testy = Testy { a: 1, b: 2 };

        assert_eq!(testy.a(), 1);
        assert_eq!(testy.b(), 2);
    }

    #[test]
    fn test_mut_clone() {
        let mut testy = Testy { a: 1, b: 2 };

        *testy.b_mut() *= -1;

        assert_eq!(testy.b(), -2);
    }
}
