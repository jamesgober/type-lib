//! Writing a custom `Validator` with a structured, domain-specific error type.
//!
//! Run with: `cargo run --example custom_rule`

use type_lib::{Refined, Validator};

/// A structured failure that tells the caller exactly what went wrong.
#[derive(Debug, PartialEq, Eq)]
enum AgeError {
    Negative,
    TooOld { max: u8, found: i16 },
}

/// A human age: 0..=130.
struct HumanAge;

impl Validator<i16> for HumanAge {
    type Error = AgeError;

    fn validate(value: &i16) -> Result<(), Self::Error> {
        match *value {
            n if n < 0 => Err(AgeError::Negative),
            n if n > 130 => Err(AgeError::TooOld { max: 130, found: n }),
            _ => Ok(()),
        }
    }
}

type Age = Refined<i16, HumanAge>;

fn main() {
    for candidate in [30, -1, 200] {
        match Age::new(candidate) {
            Ok(age) => println!("age {} accepted", age.get()),
            Err(AgeError::Negative) => println!("age {candidate} rejected: negative"),
            Err(AgeError::TooOld { max, found }) => {
                println!("age {found} rejected: exceeds max of {max}");
            }
        }
    }
}
