//! Clone macro.

/// Create an access-by-clone method for the given variable.
/// As opposed to the `access!()` macro, this will return a value by cloning it.
/// To automatically implement a getter, use it like so:
/// ```rust
/// # use Aetherus::clone;
/// pub struct DocStruct {
///     number_prop: f64,   
/// }
///
/// impl DocStruct {
///     clone!(number_prop: f64);
/// }
/// ```
/// which generates a getter at `DocStruct::number_prop()`.
///
/// ## Generation of Setter Method
/// In addition, this macro can optionally generate a 'setter' method, which
/// returns a mutable reference to the variable. To do this, the above code can
/// be augmented as below
///
/// ```rust
/// # use Aetherus::clone;
/// pub struct DocStruct {
///     number_prop: f64,
/// }
///
/// impl DocStruct {
///     clone!(number_prop, number_prop_mut: f64);
/// }
/// ```
/// which generates a setter method at `DocStruct::number_prop_mut()`.
///
/// ## Warning - Usage with `access!()`
/// As both the `access!()` and `clone!()` macros are generating getters and
/// setters with the name of the provided property, they are mutually exclusive.
/// Attempting to use both on the same property will result in a compilation
/// error.
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

    /// A simple test for immutable access with an clone-generated getter.
    #[test]
    fn test_clone() {
        let testy = Testy { a: 1, b: 2 };

        assert_eq!(testy.a(), 1);
        assert_eq!(testy.b(), 2);
    }

    /// A simple test for mutable access with an access-generated setter.
    #[test]
    fn test_mut_clone() {
        let mut testy = Testy { a: 1, b: 2 };

        *testy.b_mut() *= -1;

        assert_eq!(testy.b(), -2);
    }
}
