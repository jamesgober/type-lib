//! The [`Refined`] zero-cost wrapper around a validated value.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::Deref;

use crate::Validator;

/// A value of type `T` that is guaranteed to satisfy the validator `V`.
///
/// `Refined` is the heart of the parse-dont-validate pattern: a value is checked
/// **once**, when the wrapper is constructed, and the type then proves the
/// invariant for the rest of the value's life. Code that receives a
/// `Refined<T, V>` never has to re-validate, because a `Refined` that violates
/// `V` cannot be constructed through safe code.
///
/// The wrapper is `#[repr(transparent)]` and stores only the value plus a
/// zero-sized marker, so it has the exact same size and layout as `T`. The
/// guarantee costs nothing at runtime.
///
/// There is deliberately no `DerefMut` and no public field: handing out a `&mut
/// T` would let a caller mutate the value into an invalid state behind the
/// type's back. To change a refined value, build a new one with [`Refined::new`]
/// (or recover the inner value with [`Refined::into_inner`], change it, and
/// re-wrap it).
///
/// # Examples
///
/// Define a rule, alias a domain type, and construct it:
///
/// ```rust
/// use type_lib::{Refined, ValidationError, Validator};
///
/// struct NonEmpty;
///
/// impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
///     type Error = ValidationError;
///
///     fn validate(value: &S) -> Result<(), Self::Error> {
///         if value.as_ref().is_empty() {
///             Err(ValidationError::new("non_empty", "value must not be empty"))
///         } else {
///             Ok(())
///         }
///     }
/// }
///
/// /// A username that can never be empty.
/// type Username = Refined<String, NonEmpty>;
///
/// let user = Username::new("alice".to_owned());
/// assert!(user.is_ok());
/// assert!(Username::new(String::new()).is_err());
/// ```
///
/// Read the inner value through [`Deref`], [`Refined::get`], or
/// [`Refined::into_inner`]:
///
/// ```rust
/// # use type_lib::{Refined, ValidationError, Validator};
/// # struct NonEmpty;
/// # impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
/// #     type Error = ValidationError;
/// #     fn validate(value: &S) -> Result<(), Self::Error> {
/// #         if value.as_ref().is_empty() {
/// #             Err(ValidationError::new("non_empty", "empty"))
/// #         } else { Ok(()) }
/// #     }
/// # }
/// # type Username = Refined<String, NonEmpty>;
/// # fn main() -> Result<(), ValidationError> {
/// let user = Username::new("alice".to_owned())?;
///
/// assert_eq!(user.len(), 5);          // via Deref to String
/// assert_eq!(user.get(), "alice");    // borrow the inner value
/// assert_eq!(user.into_inner(), "alice"); // take ownership back
/// # Ok(())
/// # }
/// ```
#[repr(transparent)]
pub struct Refined<T, V: Validator<T>> {
    value: T,
    // `fn() -> V` keeps `Refined` covariant in `V` and unconditionally
    // `Send + Sync + Unpin`, since `V` is only a type-level tag and is never
    // stored or produced.
    _validator: PhantomData<fn() -> V>,
}

impl<T, V: Validator<T>> Refined<T, V> {
    /// Validates `value` and, on success, wraps it.
    ///
    /// This is the only safe way to construct a `Refined`, which is what makes
    /// the invariant trustworthy everywhere else.
    ///
    /// # Errors
    ///
    /// Returns [`V::Error`](Validator::Error) when `value` fails `V`'s rule. The
    /// value is dropped in that case.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use type_lib::{Refined, ValidationError, Validator};
    /// # struct NonEmpty;
    /// # impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
    /// #     type Error = ValidationError;
    /// #     fn validate(v: &S) -> Result<(), Self::Error> {
    /// #         if v.as_ref().is_empty() { Err(ValidationError::new("e", "empty")) } else { Ok(()) }
    /// #     }
    /// # }
    /// let ok = Refined::<&str, NonEmpty>::new("hi");
    /// assert!(ok.is_ok());
    ///
    /// let bad = Refined::<&str, NonEmpty>::new("");
    /// assert!(bad.is_err());
    /// ```
    pub fn new(value: T) -> Result<Self, V::Error> {
        V::validate(&value)?;
        Ok(Self {
            value,
            _validator: PhantomData,
        })
    }

    /// Borrows the validated inner value.
    ///
    /// The same borrow is also available through [`Deref`], so methods of `T`
    /// can be called directly on a `Refined`; `get` is the explicit form for
    /// when inference needs a nudge.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use type_lib::{Refined, ValidationError, Validator};
    /// # struct AnyI32;
    /// # impl Validator<i32> for AnyI32 {
    /// #     type Error = ValidationError;
    /// #     fn validate(_: &i32) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// let n = Refined::<i32, AnyI32>::new(7).expect("always valid");
    /// assert_eq!(*n.get(), 7);
    /// ```
    #[must_use]
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Consumes the wrapper and returns the inner value.
    ///
    /// Use this when you need to mutate or transform the value: take it out,
    /// change it, then re-wrap with [`Refined::new`] to re-establish the
    /// guarantee.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use type_lib::{Refined, ValidationError, Validator};
    /// # struct AnyI32;
    /// # impl Validator<i32> for AnyI32 {
    /// #     type Error = ValidationError;
    /// #     fn validate(_: &i32) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// let n = Refined::<i32, AnyI32>::new(7).expect("always valid");
    /// assert_eq!(n.into_inner(), 7);
    /// ```
    #[must_use]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T, V: Validator<T>> Deref for Refined<T, V> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, V: Validator<T>> AsRef<T> for Refined<T, V> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

// The trait impls below are written by hand rather than derived. Deriving would
// add a `V: Trait` bound, but `V` is a zero-sized marker that usually does not
// implement `Clone`, `PartialEq`, and friends — and need not, since it is never
// stored. Bounding only on `T` keeps `Refined` usable with any marker type.

impl<T: Clone, V: Validator<T>> Clone for Refined<T, V> {
    fn clone(&self) -> Self {
        // The value is already valid, so cloning it cannot break the invariant;
        // re-validation would be wasted work.
        Self {
            value: self.value.clone(),
            _validator: PhantomData,
        }
    }
}

impl<T: Copy, V: Validator<T>> Copy for Refined<T, V> {}

impl<T: fmt::Debug, V: Validator<T>> fmt::Debug for Refined<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Refined").field(&self.value).finish()
    }
}

impl<T: fmt::Display, V: Validator<T>> fmt::Display for Refined<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: PartialEq, V: Validator<T>> PartialEq for Refined<T, V> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq, V: Validator<T>> Eq for Refined<T, V> {}

impl<T: PartialOrd, V: Validator<T>> PartialOrd for Refined<T, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Ord, V: Validator<T>> Ord for Refined<T, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T: Hash, V: Validator<T>> Hash for Refined<T, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::ValidationError;

    struct NonEmpty;

    impl Validator<&str> for NonEmpty {
        type Error = ValidationError;

        fn validate(value: &&str) -> Result<(), Self::Error> {
            if value.is_empty() {
                Err(ValidationError::new("non_empty", "value must not be empty"))
            } else {
                Ok(())
            }
        }
    }

    struct Positive;

    impl Validator<i32> for Positive {
        type Error = ValidationError;

        fn validate(value: &i32) -> Result<(), Self::Error> {
            if *value > 0 {
                Ok(())
            } else {
                Err(ValidationError::new("positive", "value must be > 0"))
            }
        }
    }

    /// A `Clone`-but-not-`Copy` value type for exercising the `Clone` impl
    /// without pulling in `alloc`.
    #[derive(Clone, PartialEq, Debug)]
    struct Tag(i32);

    struct AnyTag;

    impl Validator<Tag> for AnyTag {
        type Error = ValidationError;

        fn validate(_value: &Tag) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn new_accepts_valid_and_rejects_invalid() {
        assert!(Refined::<&str, NonEmpty>::new("ok").is_ok());
        let err = Refined::<&str, NonEmpty>::new("").unwrap_err();
        assert_eq!(err.code(), "non_empty");
    }

    #[test]
    fn accessors_return_inner() {
        let n = Refined::<i32, Positive>::new(42).unwrap();
        assert_eq!(*n.get(), 42);
        assert_eq!(*n.as_ref(), 42);
        assert_eq!(*n, 42); // Deref
        assert_eq!(n.into_inner(), 42);
    }

    #[test]
    fn delegated_traits_compare_by_value() {
        let a = Refined::<i32, Positive>::new(1).unwrap();
        let b = Refined::<i32, Positive>::new(1).unwrap();
        let c = Refined::<i32, Positive>::new(2).unwrap();
        assert_eq!(a, b);
        assert!(a < c);
    }

    #[test]
    fn copy_inner_makes_refined_copy() {
        let a = Refined::<i32, Positive>::new(3).unwrap();
        let b = a; // Copy: `a` is still usable afterwards.
        assert_eq!(*a.get(), 3);
        assert_eq!(*b.get(), 3);
    }

    #[test]
    fn clone_inner_clones_refined_without_revalidating() {
        let a = Refined::<Tag, AnyTag>::new(Tag(7)).unwrap();
        let b = a.clone();
        assert_eq!(*a.get(), Tag(7));
        assert_eq!(*b.get(), Tag(7));
    }

    #[test]
    fn layout_is_transparent_over_t() {
        assert_eq!(
            core::mem::size_of::<Refined<i32, Positive>>(),
            core::mem::size_of::<i32>(),
        );
        assert_eq!(
            core::mem::size_of::<Refined<&str, NonEmpty>>(),
            core::mem::size_of::<&str>(),
        );
    }
}
