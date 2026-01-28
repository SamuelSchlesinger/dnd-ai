//! Proc macros for D&D tool generation.
//!
//! Provides `#[derive(Tool)]` to automatically generate JSON schemas
//! and tool trait implementations from struct definitions.
//!
//! # Example
//!
//! ```ignore
//! /// Roll dice using standard notation
//! #[derive(Tool)]
//! #[tool(name = "roll_dice")]
//! struct RollDice {
//!     /// Dice notation like "2d6+3" or "1d20"
//!     notation: String,
//!     /// Optional purpose for the roll
//!     #[tool(optional)]
//!     purpose: Option<String>,
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Field, Lit, Meta, Type};

/// Derive macro for generating Tool implementations.
///
/// # Attributes
///
/// - `#[tool(name = "...")]` - Override the tool name (defaults to snake_case struct name)
/// - `#[tool(optional)]` on fields - Mark field as optional in JSON schema
/// - `#[tool(rename = "...")]` on fields - Override field name in schema
#[proc_macro_derive(Tool, attributes(tool))]
pub fn derive_tool(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_tool(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn expand_tool(input: DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = &input.ident;

    // Get tool name from attribute or default to snake_case
    let tool_name = get_tool_name(&input)?;

    // Get description from doc comments
    let description = get_doc_comment(&input.attrs);

    // Parse fields
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(named) => &named.named,
            _ => return Err(syn::Error::new_spanned(input, "Tool derive only supports structs with named fields")),
        },
        _ => return Err(syn::Error::new_spanned(input, "Tool derive only supports structs")),
    };

    // Generate JSON schema for properties
    let mut property_tokens = Vec::new();
    let mut required_fields = Vec::new();

    for field in fields {
        let _field_ident = field.ident.as_ref().unwrap();
        let field_name_str = get_field_name(field)?;
        let field_desc = get_doc_comment(&field.attrs);
        let is_optional = is_field_optional(field)?;
        let field_type = &field.ty;

        let type_schema = type_to_schema(field_type)?;

        let desc_token = if field_desc.is_empty() {
            quote! {}
        } else {
            quote! { property["description"] = serde_json::json!(#field_desc); }
        };

        property_tokens.push(quote! {
            {
                let mut property = #type_schema;
                #desc_token
                properties.insert(#field_name_str.to_string(), property);
            }
        });

        if !is_optional && !is_option_type(field_type) {
            required_fields.push(field_name_str.clone());
        }
    }

    let required_array: Vec<_> = required_fields.iter().map(|s| quote! { #s }).collect();

    Ok(quote! {
        impl #struct_name {
            /// Get the tool name.
            pub fn tool_name() -> &'static str {
                #tool_name
            }

            /// Get the tool description.
            pub fn tool_description() -> &'static str {
                #description
            }

            /// Generate the JSON schema for this tool's input.
            pub fn input_schema() -> serde_json::Value {
                let mut properties = serde_json::Map::new();
                #(#property_tokens)*

                let required: Vec<&str> = vec![#(#required_array),*];

                serde_json::json!({
                    "type": "object",
                    "properties": properties,
                    "required": required
                })
            }

            /// Create a Tool definition for use with the Claude API.
            pub fn as_tool() -> claude::Tool {
                claude::Tool {
                    name: Self::tool_name().to_string(),
                    description: Self::tool_description().to_string(),
                    input_schema: Self::input_schema(),
                }
            }
        }
    })
}

fn get_tool_name(input: &DeriveInput) -> syn::Result<String> {
    for attr in &input.attrs {
        if attr.path().is_ident("tool") {
            let meta = attr.parse_args::<Meta>()?;
            if let Meta::NameValue(nv) = meta {
                if nv.path.is_ident("name") {
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        if let Lit::Str(s) = &expr_lit.lit {
                            return Ok(s.value());
                        }
                    }
                }
            }
        }
    }

    // Default: convert struct name to snake_case
    let name = input.ident.to_string();
    Ok(to_snake_case(&name))
}

fn get_field_name(field: &Field) -> syn::Result<String> {
    for attr in &field.attrs {
        if attr.path().is_ident("tool") {
            if let Ok(meta) = attr.parse_args::<Meta>() {
                if let Meta::NameValue(nv) = meta {
                    if nv.path.is_ident("rename") {
                        if let syn::Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(s) = &expr_lit.lit {
                                return Ok(s.value());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(field.ident.as_ref().unwrap().to_string())
}

fn is_field_optional(field: &Field) -> syn::Result<bool> {
    for attr in &field.attrs {
        if attr.path().is_ident("tool") {
            if let Ok(meta) = attr.parse_args::<Meta>() {
                if let Meta::Path(path) = meta {
                    if path.is_ident("optional") {
                        return Ok(true);
                    }
                }
            }
        }
    }
    Ok(false)
}

fn get_doc_comment(attrs: &[syn::Attribute]) -> String {
    let mut docs = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        docs.push(s.value().trim().to_string());
                    }
                }
            }
        }
    }
    docs.join(" ")
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

fn type_to_schema(ty: &Type) -> syn::Result<TokenStream2> {
    Ok(match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let ident = &segment.ident;
                let ident_str = ident.to_string();

                match ident_str.as_str() {
                    "String" | "str" => quote! { serde_json::json!({"type": "string"}) },
                    "i8" | "i16" | "i32" | "i64" | "isize" |
                    "u8" | "u16" | "u32" | "u64" | "usize" => {
                        quote! { serde_json::json!({"type": "integer"}) }
                    }
                    "f32" | "f64" => quote! { serde_json::json!({"type": "number"}) },
                    "bool" => quote! { serde_json::json!({"type": "boolean"}) },
                    "Option" => {
                        // Extract inner type
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                                return type_to_schema(inner);
                            }
                        }
                        quote! { serde_json::json!({}) }
                    }
                    "Vec" => {
                        // Extract inner type for array
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                                let inner_schema = type_to_schema(inner)?;
                                return Ok(quote! {
                                    serde_json::json!({
                                        "type": "array",
                                        "items": #inner_schema
                                    })
                                });
                            }
                        }
                        quote! { serde_json::json!({"type": "array"}) }
                    }
                    _ => {
                        // Unknown type - use object
                        quote! { serde_json::json!({"type": "object"}) }
                    }
                }
            } else {
                quote! { serde_json::json!({}) }
            }
        }
        _ => quote! { serde_json::json!({}) },
    })
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
