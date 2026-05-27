<h1 align="center">
		<img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
		<br>
		<b>type-lib</b>
		<br>
		<sub><sup>API REFERENCE</sup></sub>
</h1>
<div align="center">
		<sup>
				<a href="../README.md" title="Project Home"><b>HOME</b></a>
				<span>&nbsp;│&nbsp;</span>
				<span>API</span>
		</sup>
</div>

<br>

`type-lib` enforces domain invariants at construction and proves them through the
type system thereafter — the parse-dont-validate pattern. This document is the
complete reference for the public API as of `v0.5.0`: every exported item, what
it does, the meaning of each parameter and return value, the error semantics, and
runnable examples for each use case.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Public API](#public-api)
	- [`Validator`](#validator)
	- [`Refined`](#refined)
	- [`ValidationError`](#validationerror)
	- [Built-in rules](#built-in-rules)
		- [Length rules](#length-rules)
		- [Numeric rules](#numeric-rules)
		- [String rules](#string-rules)
	- [Combinators](#combinators)
	- [`Validated` (derive macro)](#validated-derive)
	- [`prelude`](#prelude)
	- [`VERSION`](#version)
- [Patterns](#patterns)
	- [Aliasing a domain type](#aliasing-a-domain-type)
	- [Reusing one rule across borrow forms](#reusing-one-rule-across-borrow-forms)
	- [Structured error types](#structured-error-types)
	- [Updating a refined value](#updating-a-refined-value)
- [Feature Flags](#feature-flags)
- [Semantics and Compatibility](#semantics-and-compatibility)

## Installation

```toml
[dependencies]
type-lib = "0.6.0"

# with the derive macro
type-lib = { version = "0.6.0", features = ["derive"] }
```

To build without the standard library, disable default features. The core
`Validator` / `Refined` API and all borrowed-value rules are identical; the
`alloc` feature adds owned-type (`String` / `Vec`) length rules, and `std` adds
the `std::error::Error` impl on `ValidationError`.

```toml
[dependencies]
# no_std, core API + borrowed-value rules
type-lib = { version = "0.6.0", default-features = false }

# no_std + owned-type rules
type-lib = { version = "0.6.0", default-features = false, features = ["alloc"] }
```

MSRV: Rust 1.75.

## Quick Start

Define a rule, alias a domain type, and construct it. A `Refined` that violates
its rule cannot be built through safe code, so any code holding one can trust the
invariant.

```rust
use type_lib::{Refined, ValidationError, Validator};

struct NonEmpty;

impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
    type Error = ValidationError;

    fn validate(value: &S) -> Result<(), Self::Error> {
        if value.as_ref().is_empty() {
            Err(ValidationError::new("non_empty", "value must not be empty"))
        } else {
            Ok(())
        }
    }
}

type Username = Refined<String, NonEmpty>;

let user = Username::new("alice".to_owned());
assert!(user.is_ok());
assert!(Username::new(String::new()).is_err());
```

## Public API

The complete public surface in `v0.2.0` is two traits/types for expressing rules
and validated values (`Validator`, `Refined`), one ready-made error
(`ValidationError`), the `prelude` module, and the `VERSION` constant.

### `Validator`

A reusable, type-level validation rule applied to values of type `T`.

```rust
pub trait Validator<T: ?Sized> {
    type Error;
    fn validate(value: &T) -> Result<(), Self::Error>;
}
```

**Description**

- A `Validator` is a *type-level predicate*: it is implemented on a zero-sized
  marker type and never instantiated. The rule is selected entirely through the
  type system, which is what lets [`Refined`](#refined) attach it to a value at
  no runtime cost.
- `validate` is a pure check. It borrows the value and reports success or
  failure; it never mutates or transforms the value.
- `T` is `?Sized`, so a rule can target an unsized type such as `str` or `[u8]`.

**Associated types**

- `Error` — the value returned when a value violates the rule. Choose
  [`ValidationError`](#validationerror) for simple cases, or any custom type when
  callers need structured failure details.

**Method: `validate`**

- Parameters: `value: &T` — the value to check, borrowed.
- Returns: `Ok(())` when `value` satisfies the rule; `Err(Self::Error)`
  otherwise.
- Errors: returns `Self::Error` describing the violation.

**Examples**

A simple rule using the bundled error type:

```rust
use type_lib::{ValidationError, Validator};

struct NonEmpty;

impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
    type Error = ValidationError;

    fn validate(value: &S) -> Result<(), Self::Error> {
        if value.as_ref().is_empty() {
            Err(ValidationError::new("non_empty", "value must not be empty"))
        } else {
            Ok(())
        }
    }
}

assert!(NonEmpty::validate("hello").is_ok());
assert!(NonEmpty::validate("").is_err());
```

A rule over a numeric type:

```rust
use type_lib::{ValidationError, Validator};

struct Even;

impl Validator<i64> for Even {
    type Error = ValidationError;

    fn validate(value: &i64) -> Result<(), Self::Error> {
        if value % 2 == 0 {
            Ok(())
        } else {
            Err(ValidationError::new("even", "value must be even"))
        }
    }
}

assert!(Even::validate(&4).is_ok());
assert!(Even::validate(&3).is_err());
```

A rule with a bespoke, structured error type:

```rust
use type_lib::Validator;

#[derive(Debug, PartialEq)]
struct TooLong { limit: usize, actual: usize }

struct MaxLen8;

impl Validator<str> for MaxLen8 {
    type Error = TooLong;

    fn validate(value: &str) -> Result<(), Self::Error> {
        let actual = value.chars().count();
        if actual > 8 {
            Err(TooLong { limit: 8, actual })
        } else {
            Ok(())
        }
    }
}

assert_eq!(MaxLen8::validate("far-too-long"), Err(TooLong { limit: 8, actual: 12 }));
```

### `Refined`

A value of type `T` guaranteed to satisfy the validator `V`.

```rust
pub struct Refined<T, V: Validator<T>> { /* private */ }
```

**Description**

- `Refined<T, V>` is the core parse-dont-validate type. It is constructed only by
  validating a value once, and from then on the type proves the invariant — code
  that receives a `Refined<T, V>` never re-validates.
- It is `#[repr(transparent)]` and stores only the value plus a zero-sized
  marker, so it has the exact same size and layout as `T`.
- There is no `DerefMut`, no public field, and no unchecked constructor: a
  `Refined` cannot be mutated into an invalid state behind the type's back.

**Type parameters**

- `T` — the wrapped value type (must be `Sized`).
- `V` — the [`Validator`](#validator) that vouches for the value. Usually a
  zero-sized marker type.

**Constructor: `Refined::new`**

```rust
pub fn new(value: T) -> Result<Self, V::Error>
```

- Parameters: `value: T` — the candidate value, taken by value.
- Returns: `Ok(Refined)` if `value` passes `V::validate`; otherwise
  `Err(V::Error)`. On failure the value is dropped.
- Errors: returns [`V::Error`](#validator) when the value fails the rule.

**Accessors**

- `get(&self) -> &T` — borrows the validated inner value.
- `into_inner(self) -> T` — consumes the wrapper and returns the inner value.
- Implements `Deref<Target = T>` and `AsRef<T>`, so `T`'s methods can be called
  directly on a `Refined`.

**Delegated trait impls**

`Refined<T, V>` forwards the following to the inner value, bounded only on `T`
(never on the marker `V`): `Clone` (when `T: Clone`), `Copy` (when `T: Copy`),
`Debug`, `Display`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash`. Cloning a
`Refined` does not re-validate — a valid value stays valid.

**Examples**

Construct, read, and unwrap:

```rust
use type_lib::{Refined, ValidationError, Validator};

struct Positive;

impl Validator<i32> for Positive {
    type Error = ValidationError;

    fn validate(value: &i32) -> Result<(), Self::Error> {
        if *value > 0 { Ok(()) } else {
            Err(ValidationError::new("positive", "value must be > 0"))
        }
    }
}

type Count = Refined<i32, Positive>;

let n = Count::new(7).expect("positive");
assert_eq!(*n.get(), 7);   // borrow
assert_eq!(*n, 7);         // via Deref
assert_eq!(n.into_inner(), 7); // take ownership

assert!(Count::new(0).is_err());
```

Use a refined value as a hash-map key (it delegates `Eq` + `Hash` to the inner
value):

```rust
# use type_lib::{Refined, ValidationError, Validator};
# struct Positive;
# impl Validator<i32> for Positive {
#     type Error = ValidationError;
#     fn validate(v: &i32) -> Result<(), Self::Error> {
#         if *v > 0 { Ok(()) } else { Err(ValidationError::new("positive", "p")) }
#     }
# }
use std::collections::HashMap;

type Count = Refined<i32, Positive>;

let mut counts: HashMap<Count, &str> = HashMap::new();
let _ = counts.insert(Count::new(1).expect("positive"), "one");
assert_eq!(counts.get(&Count::new(1).expect("positive")), Some(&"one"));
```

Confirm the zero-overhead layout:

```rust
# use type_lib::{Refined, ValidationError, Validator};
# struct Positive;
# impl Validator<i32> for Positive {
#     type Error = ValidationError;
#     fn validate(v: &i32) -> Result<(), Self::Error> {
#         if *v > 0 { Ok(()) } else { Err(ValidationError::new("positive", "p")) }
#     }
# }
assert_eq!(
    core::mem::size_of::<Refined<i32, Positive>>(),
    core::mem::size_of::<i32>(),
);
```

### `ValidationError`

A lightweight, `no_std`-friendly validation failure carrying a machine-readable
code and a human-readable message.

```rust
pub struct ValidationError { /* private */ }
```

**Description**

- The default error for simple validators and the recommended starting point for
  hand-written ones. It is `Copy`, holds no owned data, and never allocates.
- The `code` is a stable, lowercase identifier you can match on (`"non_empty"`,
  `"out_of_range"`); the `message` is for logs and diagnostics. Keep the code
  stable across releases even if you reword the message.
- Under the `std` feature it implements [`std::error::Error`].
- Derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, and `Hash`.
- `Display` renders as `"<code>: <message>"`.

**Methods**

- `new(code: &'static str, message: &'static str) -> Self` — a `const fn`
  constructor, so errors can be declared as `const`/`static`.
- `code(&self) -> &'static str` — the stable code (intended for matching).
- `message(&self) -> &'static str` — the human-readable message (intended for
  display, may change between releases).

**Examples**

Construct and inspect:

```rust
use type_lib::ValidationError;

let err = ValidationError::new("non_empty", "value must not be empty");
assert_eq!(err.code(), "non_empty");
assert_eq!(err.message(), "value must not be empty");
assert_eq!(err.to_string(), "non_empty: value must not be empty");
```

Declare reusable errors as constants:

```rust
use type_lib::ValidationError;

const EMPTY: ValidationError = ValidationError::new("non_empty", "value must not be empty");
const RANGE: ValidationError = ValidationError::new("out_of_range", "value out of range");

assert_eq!(EMPTY.code(), "non_empty");
assert_eq!(RANGE.code(), "out_of_range");
```

Branch on the stable code:

```rust
use type_lib::ValidationError;

fn user_message(err: &ValidationError) -> &'static str {
    match err.code() {
        "non_empty" => "the field was left blank",
        "out_of_range" => "the number is outside the allowed range",
        _ => "the value is invalid",
    }
}

let err = ValidationError::new("out_of_range", "expected 1..=10, got 42");
assert_eq!(user_message(&err), "the number is outside the allowed range");
```

### Built-in rules

The `type_lib::rules` module ships ready-made [`Validator`](#validator)
implementations for the most common invariants. Every built-in rule reports
failures as [`ValidationError`](#validationerror), so they share one error type
and compose freely with the [combinators](#combinators).

Rules are marker types: you never construct one, you name it as the `V` parameter
of a [`Refined`](#refined) or call its associated `validate` directly.

#### Length rules

Length rules operate on any type implementing `type_lib::rules::HasLength`. The
crate implements it for `str` and `[T]` always, for `String` and `Vec<T>` under
the `alloc` feature, and for shared references (`&U`) so a rule sees through one
`&`. For strings, length is the number of `char`s (Unicode scalar values); for
slices and vectors it is the element count.

| Rule | Accepts when |
|---|---|
| `NonEmpty` | length `> 0` |
| `MinLen<MIN>` | length `>= MIN` |
| `MaxLen<MAX>` | length `<= MAX` |
| `LenRange<MIN, MAX>` | `MIN <= length <= MAX` |

Error codes: `"non_empty"`, `"min_len"`, `"max_len"`, `"len_range"`.

```rust
use type_lib::rules::{LenRange, MaxLen, NonEmpty};
use type_lib::Validator;

assert!(NonEmpty::validate("hi").is_ok());
assert!(NonEmpty::validate("").is_err());

assert!(MaxLen::<5>::validate("hello").is_ok());
assert!(MaxLen::<5>::validate("too long").is_err());

// Length counts characters, not bytes.
assert_eq!("héllo".chars().count(), 5);
assert!(LenRange::<1, 5>::validate("héllo").is_ok());
```

Length rules also apply to slices:

```rust
use type_lib::rules::MinLen;
use type_lib::Validator;

assert!(MinLen::<2>::validate(&[1, 2, 3][..]).is_ok());
assert!(MinLen::<2>::validate(&[1][..]).is_err());
```

#### Numeric rules

Sign rules apply to the signed integer and floating-point primitives.
[`InRange`](#numeric-rules) applies to the integer primitives that fit losslessly
in an `i64` (`i8`/`i16`/`i32`/`i64`, `u8`/`u16`/`u32`) and takes `i64` const
bounds.

| Rule | Accepts when |
|---|---|
| `Positive` | `value > 0` |
| `NonNegative` | `value >= 0` |
| `Negative` | `value < 0` |
| `NonPositive` | `value <= 0` |
| `InRange<MIN, MAX>` | `MIN <= value <= MAX` (inclusive) |

Error codes: `"positive"`, `"non_negative"`, `"negative"`, `"non_positive"`,
`"in_range"`.

```rust
use type_lib::rules::{InRange, Positive};
use type_lib::Validator;

assert!(Positive::validate(&3_i32).is_ok());
assert!(Positive::validate(&0_i32).is_err());
assert!(Positive::validate(&-1.5_f64).is_err());

// A percentage.
assert!(InRange::<0, 100>::validate(&50_u8).is_ok());
assert!(InRange::<0, 100>::validate(&150_i32).is_err());

// Ranges may be negative.
assert!(InRange::<-10, 10>::validate(&-5_i16).is_ok());
```

#### String rules

String rules apply to any `S: AsRef<str>` (so `&str`, `String`, and borrowed
string types). They check character content only; combine with a length rule via
[`And`](#combinators) when you also need a length bound.

| Rule | Accepts when |
|---|---|
| `Ascii` | every character is ASCII |
| `Alphanumeric` | every character is alphanumeric (Unicode) |
| `Trimmed` | no leading or trailing whitespace |

Error codes: `"ascii"`, `"alphanumeric"`, `"trimmed"`. An empty string passes
`Ascii` and `Alphanumeric` vacuously.

```rust
use type_lib::rules::{Alphanumeric, Ascii, Trimmed};
use type_lib::Validator;

assert!(Ascii::validate("plain-text_123").is_ok());
assert!(Ascii::validate("café").is_err());

assert!(Alphanumeric::validate("abc123").is_ok());
assert!(Alphanumeric::validate("user_name").is_err()); // underscore

assert!(Trimmed::validate("clean").is_ok());
assert!(Trimmed::validate(" padded ").is_err());
```

### Combinators

The `type_lib::combinator` module composes [`Validator`](#validator)s at the type
level. Each combinator is itself a `Validator`, so they nest:
`And<A, Or<B, C>>` is a valid rule.

| Combinator | Passes when | Error type |
|---|---|---|
| `And<A, B>` | both `A` and `B` pass (`A` checked first) | shared `A::Error` |
| `Or<A, B>` | `A` or `B` passes (`A` checked first) | shared `A::Error` |
| `Not<A>` | `A` fails | [`ValidationError`](#validationerror) (code `"not"`) |

`And` and `Or` require their two sub-rules to share one [`Validator::Error`] type
— which every built-in rule satisfies (all report `ValidationError`). When both
sides of an `Or` fail, the error returned is the second rule's. `Not` accepts any
sub-rule regardless of its error type.

```rust
use type_lib::combinator::{And, Not, Or};
use type_lib::rules::{Alphanumeric, Ascii, MaxLen, NonEmpty};
use type_lib::Validator;

// Both must hold.
type ShortNonEmpty = And<NonEmpty, MaxLen<8>>;
assert!(ShortNonEmpty::validate("ok").is_ok());
assert!(ShortNonEmpty::validate("").is_err());
assert!(ShortNonEmpty::validate("way too long").is_err());

// Either may hold.
type AlnumOrAscii = Or<Alphanumeric, Ascii>;
assert!(AlnumOrAscii::validate("abc123").is_ok());

// Inversion: require a value that is *not* pure ASCII.
type NonAscii = Not<Ascii>;
assert!(NonAscii::validate("café").is_ok());
assert!(NonAscii::validate("plain").is_err());
```

Wrapped in a `Refined`, a composed rule reads as a single domain type:

```rust
use type_lib::combinator::And;
use type_lib::rules::{Ascii, LenRange};
use type_lib::Refined;

// An API key: 16–64 ASCII characters.
type ApiKey = Refined<String, And<Ascii, LenRange<16, 64>>>;

assert!(ApiKey::new("sk_live_0123456789abcdef".to_owned()).is_ok());
assert!(ApiKey::new("short".to_owned()).is_err());
```

### `Validated` (derive macro)

Generates a named validated newtype. Requires the `derive` feature.

```rust
#[derive(Validated)]
#[valid(/* a Validator type */)]
struct Name(/* field type */);
```

**Description**

- Apply `#[derive(Validated)]` to a **single-field tuple struct** and annotate it
  with `#[valid(<Validator>)]`, where `<Validator>` is any type implementing
  [`Validator`](#validator) for the field type — a [built-in rule](#built-in-rules),
  a [combinator](#combinators), or your own.
- Unlike a `type` alias for [`Refined`](#refined), this produces a distinct,
  nominal type with its own name, constructor, and trait impls.
- The inner field stays private, so the only way to construct the type from
  outside its module is through the generated `new` — which is what keeps the
  invariant trustworthy.

**Generated items**

- `fn new(value: T) -> Result<Self, <V as Validator<T>>::Error>` — validates and
  wraps, or returns the validator's error.
- `fn get(&self) -> &T` and `fn into_inner(self) -> T`.
- `Deref<Target = T>` and `AsRef<T>`.

Add ordinary derives (`Debug`, `Clone`, `PartialEq`, …) alongside `Validated` as
usual.

**Compile errors**

The derive reports a clear error if applied to anything other than a single-field
tuple struct, if the type is generic, or if `#[valid(...)]` is missing or
duplicated.

**Examples**

A string newtype composed from built-in rules:

```rust
use type_lib::combinator::And;
use type_lib::rules::{LenRange, Trimmed};
use type_lib::Validated;

#[derive(Validated, Debug, Clone, PartialEq)]
#[valid(And<Trimmed, LenRange<3, 16>>)]
struct Username(String);

let user = Username::new("alice".to_owned()).expect("valid");
assert_eq!(user.get(), "alice");
assert_eq!(&*user, "alice"); // via Deref

assert!(Username::new("ab".to_owned()).is_err());
assert!(Username::new("  alice  ".to_owned()).is_err());
```

A numeric newtype:

```rust
use type_lib::rules::InRange;
use type_lib::Validated;

#[derive(Validated, Debug)]
#[valid(InRange<0, 100>)]
struct Percent(i32);

assert_eq!(Percent::new(50).expect("in range").into_inner(), 50);
assert!(Percent::new(150).is_err());
```

A newtype using a custom validator with a structured error:

```rust
use type_lib::{Validated, Validator};

#[derive(Debug, PartialEq)]
struct NotEven;

struct Even;
impl Validator<i64> for Even {
    type Error = NotEven;
    fn validate(value: &i64) -> Result<(), Self::Error> {
        if value % 2 == 0 { Ok(()) } else { Err(NotEven) }
    }
}

#[derive(Validated)]
#[valid(Even)]
struct EvenNumber(i64);

assert!(EvenNumber::new(4).is_ok());
assert_eq!(EvenNumber::new(5).unwrap_err(), NotEven);
```

### `prelude`

Convenience re-exports of the foundation types and combinators.

```rust
pub use type_lib::prelude::*; // Refined, Validator, ValidationError, And, Or, Not
```

**Description**

- Glob-import to bring [`Refined`](#refined), [`Validator`](#validator),
  [`ValidationError`](#validationerror), and the [combinators](#combinators)
  [`And`](#combinators) / `Or` / `Not` into scope in one line. The built-in
  [rules](#built-in-rules) are intentionally left out to keep the namespace
  small; import the ones you need from `type_lib::rules`.

**Example**

```rust
use type_lib::prelude::*;          // Refined, Validator, ValidationError, And, Or, Not
use type_lib::rules::{Ascii, NonEmpty};

// `And` comes from the prelude; the rules from `type_lib::rules`.
type Token<'a> = Refined<&'a str, And<NonEmpty, Ascii>>;

assert!(Token::new("abc123").is_ok());
assert!(Token::new("").is_err());
```

### `VERSION`

The crate version embedded by Cargo at compile time.

```rust
pub const VERSION: &str
```

**Description**

- A `&'static str` equal to the crate's `Cargo.toml` version. Useful for
  diagnostics, startup banners, and tests. Available in both `std` and `no_std`
  builds.

**Examples**

```rust
assert_eq!(type_lib::VERSION, env!("CARGO_PKG_VERSION"));
```

```rust
let banner = format!("type-lib {}", type_lib::VERSION);
assert!(banner.starts_with("type-lib "));
```

## Patterns

### Aliasing a domain type

A `type` alias gives a refined type a meaningful name and a single place to refer
to it:

```rust
use type_lib::{Refined, ValidationError, Validator};

struct NonEmpty;

impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
    type Error = ValidationError;

    fn validate(value: &S) -> Result<(), Self::Error> {
        if value.as_ref().is_empty() {
            Err(ValidationError::new("non_empty", "value must not be empty"))
        } else {
            Ok(())
        }
    }
}

type Username = Refined<String, NonEmpty>;

fn greet(user: &Username) -> String {
    format!("hello, {}", user.get())
}

let user = Username::new("alice".to_owned()).expect("non-empty");
assert_eq!(greet(&user), "hello, alice");
```

### Reusing one rule across borrow forms

Writing a rule over `S: AsRef<str>` lets the single marker type validate `&str`,
`String`, and `str` alike:

```rust
use type_lib::{Refined, ValidationError, Validator};

struct NonEmpty;

impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
    type Error = ValidationError;

    fn validate(value: &S) -> Result<(), Self::Error> {
        if value.as_ref().is_empty() {
            Err(ValidationError::new("non_empty", "value must not be empty"))
        } else {
            Ok(())
        }
    }
}

let borrowed: Result<Refined<&str, NonEmpty>, _> = Refined::new("ok");
let owned: Result<Refined<String, NonEmpty>, _> = Refined::new("ok".to_owned());
assert!(borrowed.is_ok());
assert!(owned.is_ok());
```

### Structured error types

When callers need to inspect *why* validation failed, return a custom error type
rather than `ValidationError`:

```rust
use type_lib::{Refined, Validator};

#[derive(Debug, PartialEq)]
enum AgeError { Negative, TooOld(u8) }

struct HumanAge;

impl Validator<i16> for HumanAge {
    type Error = AgeError;

    fn validate(value: &i16) -> Result<(), Self::Error> {
        match *value {
            n if n < 0 => Err(AgeError::Negative),
            n if n > 130 => Err(AgeError::TooOld(n as u8)),
            _ => Ok(()),
        }
    }
}

type Age = Refined<i16, HumanAge>;

assert!(Age::new(30).is_ok());
assert_eq!(Age::new(-1).unwrap_err(), AgeError::Negative);
```

### Updating a refined value

Because there is no `DerefMut`, you change a refined value by taking the inner
value out, modifying it, and re-wrapping — which re-establishes the guarantee:

```rust
use type_lib::{Refined, ValidationError, Validator};

struct Positive;

impl Validator<i32> for Positive {
    type Error = ValidationError;

    fn validate(value: &i32) -> Result<(), Self::Error> {
        if *value > 0 { Ok(()) } else {
            Err(ValidationError::new("positive", "value must be > 0"))
        }
    }
}

type Count = Refined<i32, Positive>;

let count = Count::new(5).expect("positive");
let bumped = Count::new(count.into_inner() + 1).expect("still positive");
assert_eq!(*bumped, 6);
```

## Feature Flags

### `std` (default)

- Enabled by default; implies `alloc`.
- Provides the [`std::error::Error`] implementation for
  [`ValidationError`](#validationerror).

### `alloc`

- Enables the length rules ([`NonEmpty`](#length-rules), `MinLen`, `MaxLen`,
  `LenRange`) for owned `String` and `Vec<T>` values, via `HasLength` impls.
- Implied by `std`. Enable it alone for `no_std` targets that have an allocator.

### `derive`

- Enables the [`Validated`](#validated-derive) derive macro (from the companion
  `type-lib-derive` crate). Off by default.

With no features (`default-features = false`), the crate is `no_std`: the
`Validator` / `Refined` / `ValidationError` API and every borrowed-value rule
(`&str`, `[T]`, numeric, string) are available; only owned-type length rules and
the `std::error::Error` impl are gated off.

## Semantics and Compatibility

- The public API surface established in `v0.2.0` is what `1.0` will preserve;
  `v0.5.0` added the [rules](#built-in-rules) and [combinators](#combinators) and
  `v0.6.0` the [`Validated`](#validated-derive) derive, all additively.
- A `Refined<T, V>` can only be constructed by passing its validator: there is no
  unchecked constructor in the public API, so the invariant holds for every
  safely constructed value.
- `Refined` is `#[repr(transparent)]` over `T`; its size and alignment match `T`.
- Every built-in rule reports [`ValidationError`](#validationerror), so they
  compose under [`And`](#combinators) / `Or` without error-type juggling.
- A derive macro for generating validated newtypes is planned for a later
  milestone. It will be additive and will not break this surface.
