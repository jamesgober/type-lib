//! Derive macro for [`type-lib`](https://docs.rs/type-lib).
//!
//! This crate provides the [`Validated`] derive. It is re-exported from
//! `type-lib` behind the `derive` feature; depend on `type-lib` with that feature
//! rather than on this crate directly:
//!
//! ```toml
//! type-lib = { version = "0.6", features = ["derive"] }
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unused_must_use)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

/// Turns a single-field newtype into a validated parse-dont-validate type.
///
/// Apply `#[derive(Validated)]` to a one-field tuple struct and annotate it with
/// `#[valid(<Validator>)]`, where `<Validator>` is any type implementing
/// [`Validator`](https://docs.rs/type-lib/latest/type_lib/trait.Validator.html)
/// for the field's type — a built-in rule, a combinator, or your own.
///
/// The derive generates, on the struct:
///
/// - `fn new(value: T) -> Result<Self, <V as Validator<T>>::Error>` — validates
///   `value` and wraps it, or returns the validator's error.
/// - `fn get(&self) -> &T` and `fn into_inner(self) -> T`.
/// - `Deref<Target = T>` and `AsRef<T>`.
///
/// The inner field stays private, so the only way to build the type from outside
/// its module is through `new` — which is what makes the invariant trustworthy.
/// Add ordinary derives (`Debug`, `Clone`, `PartialEq`, …) alongside as usual.
///
/// # Example
///
/// ```ignore
/// use type_lib::combinator::And;
/// use type_lib::rules::{LenRange, Trimmed};
/// use type_lib::Validated;
///
/// #[derive(Validated)]
/// #[valid(And<Trimmed, LenRange<3, 16>>)]
/// pub struct Username(String);
///
/// let user = Username::new("alice".to_owned()).unwrap();
/// assert_eq!(user.get(), "alice");
/// assert!(Username::new("  ".to_owned()).is_err());
/// ```
///
/// (The example is `ignore`d here because this crate cannot depend on `type-lib`;
/// the compiled version runs from `type-lib` itself.)
///
/// # Compile errors
///
/// The derive reports a clear error when applied to anything other than a
/// single-field tuple struct, when generics are present, or when the
/// `#[valid(...)]` attribute is missing or duplicated.
#[proc_macro_derive(Validated, attributes(valid))]
pub fn derive_validated(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;

    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "#[derive(Validated)] does not support generic types; use a concrete newtype",
        ));
    }

    let field_ty = single_field_type(input)?;
    let validator = validator_type(input)?;

    Ok(quote! {
        impl #name {
            /// Validates `value` against the configured rule and, on success,
            /// wraps it.
            ///
            /// # Errors
            ///
            /// Returns the validator's error when `value` does not satisfy the
            /// rule.
            pub fn new(
                value: #field_ty,
            ) -> ::core::result::Result<
                Self,
                <#validator as ::type_lib::Validator<#field_ty>>::Error,
            > {
                <#validator as ::type_lib::Validator<#field_ty>>::validate(&value)?;
                ::core::result::Result::Ok(#name(value))
            }

            /// Borrows the validated inner value.
            #[must_use]
            pub fn get(&self) -> &#field_ty {
                &self.0
            }

            /// Consumes the wrapper and returns the inner value.
            #[must_use]
            pub fn into_inner(self) -> #field_ty {
                self.0
            }
        }

        impl ::core::ops::Deref for #name {
            type Target = #field_ty;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ::core::convert::AsRef<#field_ty> for #name {
            fn as_ref(&self) -> &#field_ty {
                &self.0
            }
        }
    })
}

/// Extracts the type of the single tuple field, or errors.
fn single_field_type(input: &DeriveInput) -> syn::Result<Type> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                Ok(fields.unnamed[0].ty.clone())
            }
            _ => Err(syn::Error::new_spanned(
                &input.ident,
                "#[derive(Validated)] requires a tuple struct with exactly one field, \
                 e.g. `struct Name(String);`",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "#[derive(Validated)] can only be applied to structs",
        )),
    }
}

/// Parses the validator type from the `#[valid(...)]` attribute, or errors.
fn validator_type(input: &DeriveInput) -> syn::Result<Type> {
    let mut found: Option<Type> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("valid") {
            if found.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "duplicate #[valid(...)] attribute; specify exactly one",
                ));
            }
            found = Some(attr.parse_args::<Type>()?);
        }
    }
    found.ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            "#[derive(Validated)] requires a #[valid(<Validator>)] attribute, \
             e.g. `#[valid(NonEmpty)]`",
        )
    })
}
