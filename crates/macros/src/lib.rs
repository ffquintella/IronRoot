//! # ironroot-macros
//!
//! Procedural macros for the IronRoot framework.
//!
//! ## Available macros
//!
//! | Macro | Description |
//! |---|---|
//! | `#[derive(Entity)]` | Derives [`ironroot_core::Entity`] for a struct with an `id` field. |
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ironroot_macros::Entity;
//!
//! #[derive(Entity)]
//! struct User {
//!     id: u64,
//!     name: String,
//! }
//! ```
//!
//! The macro above expands to an `impl ironroot_core::Entity for User` block
//! that delegates `id()` to the `id` field.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derives [`ironroot_core::Entity`] for a struct that contains an `id` field.
///
/// # Requirements
///
/// - The annotated item must be a `struct`.
/// - The struct must have a field named `id`.
/// - The type of `id` must implement `Eq + std::fmt::Debug`.
///
/// # Generated code
///
/// For a struct such as:
///
/// ```rust,ignore
/// #[derive(Entity)]
/// struct User {
///     id: u64,
///     name: String,
/// }
/// ```
///
/// The macro generates:
///
/// ```rust,ignore
/// impl ironroot_core::Entity for User {
///     type Id = u64;
///     fn id(&self) -> &Self::Id { &self.id }
/// }
/// ```
///
/// # Panics
///
/// Panics at compile time if the annotated item is not a named-field struct or
/// if no `id` field is found.
#[proc_macro_derive(Entity)]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract the type of the `id` field.
    let id_type = match &input.data {
        syn::Data::Struct(data) => {
            data.fields
                .iter()
                .find(|f| f.ident.as_ref().map(|i| i == "id").unwrap_or(false))
                .map(|f| f.ty.clone())
                .unwrap_or_else(|| panic!("#[derive(Entity)] requires a field named `id`"))
        }
        _ => panic!("#[derive(Entity)] can only be applied to structs"),
    };

    let expanded = quote! {
        impl ironroot_core::Entity for #name {
            type Id = #id_type;

            fn id(&self) -> &Self::Id {
                &self.id
            }
        }
    };

    TokenStream::from(expanded)
}
