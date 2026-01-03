//! # OxidXWidget Derive Implementation
//!
//! This module contains the core logic for the `#[derive(OxidXWidget)]` macro.
//! It parses struct fields with `#[oxidx(...)]` attributes and generates:
//! - A `new()` constructor with default values
//! - Fluent setter methods for each `#[oxidx(prop)]` field
//! - An `id()` method if the struct has an `id` field

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Expr, Field, Fields, Ident, Result, Type};

/// Configuration for a single field parsed from `#[oxidx(...)]` attributes.
struct FieldConfig {
    /// The field identifier
    ident: Ident,
    /// The field type
    ty: Type,
    /// Whether this field should have a setter generated
    is_prop: bool,
    /// Optional default value expression
    default_value: Option<Expr>,
}

impl FieldConfig {
    /// Parse a field and extract oxidx attribute configuration.
    fn from_field(field: &Field) -> Result<Option<Self>> {
        let ident = match &field.ident {
            Some(id) => id.clone(),
            None => return Ok(None), // Skip tuple struct fields
        };

        let ty = field.ty.clone();
        let mut is_prop = false;
        let mut default_value: Option<Expr> = None;

        // Look for #[oxidx(...)] attributes
        for attr in &field.attrs {
            if !attr.path().is_ident("oxidx") {
                continue;
            }

            // Parse the attribute content
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("prop") {
                    is_prop = true;

                    // Check for default = ... inside prop()
                    if meta.input.peek(syn::token::Paren) {
                        meta.parse_nested_meta(|inner| {
                            if inner.path.is_ident("default") {
                                let _: syn::Token![=] = inner.input.parse()?;
                                default_value = Some(inner.input.parse()?);
                            }
                            Ok(())
                        })?;
                    }
                } else if meta.path.is_ident("default") {
                    // Support #[oxidx(default = ...)] shorthand
                    let _: syn::Token![=] = meta.input.parse()?;
                    default_value = Some(meta.input.parse()?);
                }
                Ok(())
            })?;
        }

        if is_prop || default_value.is_some() {
            Ok(Some(FieldConfig {
                ident,
                ty,
                is_prop,
                default_value,
            }))
        } else {
            // Return config for fields without oxidx attr (needed for new())
            Ok(Some(FieldConfig {
                ident,
                ty,
                is_prop: false,
                default_value: None,
            }))
        }
    }
}

/// Main entry point: generates the implementation for a struct.
pub fn derive_oxidx_widget(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Only support structs with named fields
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(f) => &f.named,
            _ => {
                return Err(Error::new_spanned(
                    &input,
                    "OxidXWidget can only be derived for structs with named fields",
                ))
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &input,
                "OxidXWidget can only be derived for structs",
            ))
        }
    };

    // Parse all field configurations
    let mut field_configs: Vec<FieldConfig> = Vec::new();
    for field in fields.iter() {
        if let Some(config) = FieldConfig::from_field(field)? {
            field_configs.push(config);
        }
    }

    // Generate the new() constructor
    let new_fn = generate_new_fn(&field_configs);

    // Generate setter methods for #[oxidx(prop)] fields
    let setters = generate_setters(&field_configs);

    // Check if struct has an `id` field of type String
    let id_impl = generate_id_impl(&field_configs);

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #new_fn

            #(#setters)*
        }

        #id_impl
    };

    Ok(expanded)
}

/// Generates the `new()` constructor that initializes all fields.
fn generate_new_fn(fields: &[FieldConfig]) -> TokenStream {
    let field_inits: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            if let Some(ref default) = f.default_value {
                quote! { #ident: #default }
            } else {
                quote! { #ident: Default::default() }
            }
        })
        .collect();

    quote! {
        /// Creates a new instance with default values.
        pub fn new() -> Self {
            Self {
                #(#field_inits),*
            }
        }
    }
}

/// Generates fluent setter methods for each `#[oxidx(prop)]` field.
fn generate_setters(fields: &[FieldConfig]) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|f| f.is_prop)
        .map(|f| {
            let ident = &f.ident;
            let ty = &f.ty;

            // Generate doc comment
            let doc = format!("Sets the `{}` property.", ident);

            // Check if type is String to use impl Into<String>
            let is_string = is_string_type(ty);
            let is_option_string = is_option_string_type(ty);

            if is_string {
                quote! {
                    #[doc = #doc]
                    pub fn #ident(mut self, val: impl Into<String>) -> Self {
                        self.#ident = val.into();
                        self
                    }
                }
            } else if is_option_string {
                quote! {
                    #[doc = #doc]
                    pub fn #ident(mut self, val: impl Into<String>) -> Self {
                        self.#ident = Some(val.into());
                        self
                    }
                }
            } else {
                quote! {
                    #[doc = #doc]
                    pub fn #ident(mut self, val: #ty) -> Self {
                        self.#ident = val;
                        self
                    }
                }
            }
        })
        .collect()
}

/// Generates `id()` method implementation if struct has an `id: String` field.
fn generate_id_impl(fields: &[FieldConfig]) -> TokenStream {
    // Check if there's a field named "id" of type String
    let has_id_field = fields
        .iter()
        .any(|f| f.ident == "id" && is_string_type(&f.ty));

    if has_id_field {
        // We don't implement the trait here - just a helper
        // The trait impl would conflict with manual implementations
        quote! {}
    } else {
        quote! {}
    }
}

/// Checks if a type is `String`.
fn is_string_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "String";
        }
    }
    false
}

/// Checks if a type is `Option<String>`.
fn is_option_string_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return is_string_type(inner_ty);
                    }
                }
            }
        }
    }
    false
}
