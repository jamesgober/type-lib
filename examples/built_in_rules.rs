//! A tour of the built-in rules across length, numeric, and string categories.
//!
//! Run with: `cargo run --example built_in_rules`

use type_lib::rules::{Alphanumeric, Ascii, InRange, MaxLen, NonEmpty, Positive, Trimmed};
use type_lib::{Refined, ValidationError, Validator};

fn report<T: ?Sized, V: Validator<T, Error = ValidationError>>(label: &str, value: &T) {
    let verdict = match V::validate(value) {
        Ok(()) => "ok".to_owned(),
        Err(err) => format!("rejected ({})", err.code()),
    };
    println!("{label:<28} {verdict}");
}

fn main() {
    println!("-- length --");
    report::<str, NonEmpty>("NonEmpty(\"hi\")", "hi");
    report::<str, NonEmpty>("NonEmpty(\"\")", "");
    report::<str, MaxLen<5>>("MaxLen<5>(\"hello\")", "hello");
    report::<str, MaxLen<5>>("MaxLen<5>(\"too long\")", "too long");

    println!("\n-- numeric --");
    report::<i32, Positive>("Positive(7)", &7);
    report::<i32, Positive>("Positive(-1)", &-1);
    report::<i32, InRange<0, 100>>("InRange<0,100>(50)", &50);
    report::<i32, InRange<0, 100>>("InRange<0,100>(150)", &150);

    println!("\n-- string --");
    report::<str, Ascii>("Ascii(\"plain\")", "plain");
    report::<str, Ascii>("Ascii(\"café\")", "café");
    report::<str, Alphanumeric>("Alphanumeric(\"abc123\")", "abc123");
    report::<str, Trimmed>("Trimmed(\" padded \")", " padded ");

    // Built once, the rule guarantees the invariant from then on.
    let port: Refined<u16, InRange<1, 65_535>> = Refined::new(8080).expect("8080 is a valid port");
    println!("\nvalidated port: {}", port.get());
}
