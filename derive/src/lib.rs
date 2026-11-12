//! Derive macro for [`type-lib`](https://docs.rs/type-lib).
//!
//! This crate provides the [`Validated`] derive. It is re-exported from
//! `type-lib` behind the `derive` feature; depend on `type-lib` with that feature
//! rather than on this crate directly:
//!
//! ```toml
//! type-lib = { version = "1.0", features = ["derive"] }
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
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Path, Token, Type, Visibility};

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
/// It also implements `TryFrom<T>` by calling `new`, so the type works with
/// `#[serde(try_from = "T")]` and with `value.try_into()`.
///
/// The field must stay private (no `pub`, `pub(crate)`, or `pub(super)`); the
/// derive refuses to compile otherwise. A visible field would let other code
/// build or change the value without `new`, so the invariant would not hold.
///
/// Derives that are safe next to `Validated`: `Debug`, `Clone`, `Copy`,
/// `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, and `serde::Serialize`. Do not
/// derive `Default`, `serde::Deserialize`, or any other derive that builds the
/// struct field by field: those skip `new`. To deserialize, add
/// `#[serde(try_from = "T")]` next to `#[derive(Deserialize)]`, which routes every
/// value through `new`.
///
/// The invariant is checked once, in `new`. If `T` allows mutation through `&T`
/// (`Cell`, `RefCell`, atomics, `Mutex`, or a type holding one), the value can
/// change after validation, so use such types only when that is acceptable.
///
/// # Options
///
/// `#[valid(V, crate = path::to::type_lib)]` sets the path the generated code
/// uses for `type-lib`. The default is `::type_lib`. Set it when `type-lib` is
/// renamed in `Cargo.toml` or reached through a re-export.
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
/// single-field tuple struct, when generics are present, when the field is not
/// private, or when the `#[valid(...)]` attribute is missing or duplicated.
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

    let (field_ty, visibility_error) = single_field_type(input)?;
    let ValidArgs { validator, krate } = valid_args(input)?;
    let visibility_error = visibility_error.map(syn::Error::into_compile_error);

    Ok(quote! {
        #visibility_error

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
                <#validator as #krate::Validator<#field_ty>>::Error,
            > {
                <#validator as #krate::Validator<#field_ty>>::validate(&value)?;
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

        impl ::core::convert::TryFrom<#field_ty> for #name {
            type Error = <#validator as #krate::Validator<#field_ty>>::Error;

            /// Validates `value` through `new`.
            fn try_from(value: #field_ty) -> ::core::result::Result<Self, Self::Error> {
                #name::new(value)
            }
        }
    })
}

/// Extracts the type of the single tuple field, or errors. The second value is
/// an error to report when the field is not private; the impls are still
/// generated in that case so the only error the user sees is that one.
fn single_field_type(input: &DeriveInput) -> syn::Result<(Type, Option<syn::Error>)> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field = &fields.unnamed[0];
                let visibility_error = if is_private(&field.vis) {
                    None
                } else {
                    Some(syn::Error::new_spanned(
                        &field.vis,
                        "#[derive(Validated)] requires the field to be private: \
                         remove the visibility so that `new` is the only way to \
                         build the type and the invariant holds",
                    ))
                };
                Ok((field.ty.clone(), visibility_error))
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

/// Returns `true` for a private field: no visibility, or `pub(self)`.
fn is_private(vis: &Visibility) -> bool {
    match vis {
        Visibility::Inherited => true,
        Visibility::Restricted(r) => r.in_token.is_none() && r.path.is_ident("self"),
        Visibility::Public(_) => false,
    }
}

/// The parsed contents of `#[valid(...)]`.
struct ValidArgs {
    validator: Type,
    krate: Path,
}

/// The arguments of one `#[valid(...)]` attribute: the validator type and an
/// optional `crate = path`, in either order.
struct RawArgs {
    validator: Option<Type>,
    krate: Option<Path>,
}

impl Parse for RawArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = RawArgs {
            validator: None,
            krate: None,
        };
        while !input.is_empty() {
            if input.peek(Token![crate]) && input.peek2(Token![=]) {
                let key: Token![crate] = input.parse()?;
                let _eq: Token![=] = input.parse()?;
                if args.krate.is_some() {
                    return Err(syn::Error::new_spanned(
                        key,
                        "duplicate `crate = ...` option in #[valid(...)]",
                    ));
                }
                args.krate = Some(input.call(Path::parse_mod_style)?);
            } else {
                let ty: Type = input.parse()?;
                if args.validator.is_some() {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "#[valid(...)] takes exactly one validator type",
                    ));
                }
                args.validator = Some(ty);
            }
            if input.is_empty() {
                break;
            }
            let _comma: Token![,] = input.parse()?;
        }
        Ok(args)
    }
}

/// Parses the validator type and options from the `#[valid(...)]` attribute,
/// or errors.
fn valid_args(input: &DeriveInput) -> syn::Result<ValidArgs> {
    let mut found: Option<(&syn::Attribute, RawArgs)> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("valid") {
            if found.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "duplicate #[valid(...)] attribute; specify exactly one",
                ));
            }
            found = Some((attr, attr.parse_args::<RawArgs>()?));
        }
    }
    let Some((attr, raw)) = found else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "#[derive(Validated)] requires a #[valid(<Validator>)] attribute, \
             e.g. `#[valid(NonEmpty)]`",
        ));
    };
    let Some(validator) = raw.validator else {
        return Err(syn::Error::new_spanned(
            attr,
            "#[valid(...)] requires a validator type, e.g. `#[valid(NonEmpty)]`",
        ));
    };
    let krate = raw.krate.unwrap_or_else(|| syn::parse_quote!(::type_lib));
    Ok(ValidArgs { validator, krate })
}
