//! Property-based tests for rule and combinator invariants.
//!
//! Each property pins the const-generic parameters at compile time and asserts
//! that the rule's verdict matches an independent reference predicate over a wide
//! range of generated inputs.

use proptest::prelude::*;
use type_lib::combinator::{And, Not, Or};
use type_lib::rules::{InRange, MaxLen, MinLen, NonEmpty};
use type_lib::{Refined, Validator};

/// Whether a rule accepts a value, as a plain boolean.
fn accepts<T: ?Sized, V: Validator<T>>(value: &T) -> bool {
    V::validate(value).is_ok()
}

proptest! {
    #[test]
    fn non_empty_matches_char_count(s in ".*") {
        let expected = s.chars().count() > 0;
        prop_assert_eq!(accepts::<str, NonEmpty>(&s), expected);
    }

    #[test]
    fn max_len_matches_char_count(s in ".*") {
        const N: usize = 10;
        let expected = s.chars().count() <= N;
        prop_assert_eq!(accepts::<str, MaxLen<N>>(&s), expected);
    }

    #[test]
    fn min_len_matches_char_count(s in ".*") {
        const N: usize = 4;
        let expected = s.chars().count() >= N;
        prop_assert_eq!(accepts::<str, MinLen<N>>(&s), expected);
    }

    #[test]
    fn in_range_matches_contains(v in any::<i32>()) {
        let expected = (0..=100).contains(&v);
        prop_assert_eq!(accepts::<i32, InRange<0, 100>>(&v), expected);
    }

    #[test]
    fn refined_new_succeeds_iff_rule_accepts(v in any::<i32>()) {
        let result = Refined::<i32, InRange<0, 100>>::new(v);
        prop_assert_eq!(result.is_ok(), accepts::<i32, InRange<0, 100>>(&v));
        if let Ok(refined) = result {
            // A successfully refined value round-trips unchanged.
            prop_assert_eq!(*refined.get(), v);
        }
    }

    #[test]
    fn and_is_conjunction(s in ".*") {
        type A = NonEmpty;
        type B = MaxLen<8>;
        let expected = accepts::<str, A>(&s) && accepts::<str, B>(&s);
        prop_assert_eq!(accepts::<str, And<A, B>>(&s), expected);
    }

    #[test]
    fn or_is_disjunction(s in ".*") {
        type A = MinLen<3>;
        type B = MaxLen<1>;
        let expected = accepts::<str, A>(&s) || accepts::<str, B>(&s);
        prop_assert_eq!(accepts::<str, Or<A, B>>(&s), expected);
    }

    #[test]
    fn not_is_negation(s in ".*") {
        type A = NonEmpty;
        let expected = !accepts::<str, A>(&s);
        prop_assert_eq!(accepts::<str, Not<A>>(&s), expected);
    }
}
