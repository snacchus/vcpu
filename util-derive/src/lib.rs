extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::quote;

fn get_enum_variants(
    input: &syn::DeriveInput,
) -> &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma> {
    let variants = match &input.data {
        syn::Data::Enum(enum_item) => &enum_item.variants,
        _ => panic!("Input must be an enum."),
    };

    assert!(
        variants.iter().all(|v| v.fields.is_empty()),
        "All variants must have no fields."
    );

    variants
}

#[proc_macro_derive(InteropGetName)]
pub fn interop_get_name_derive(input: TokenStream) -> TokenStream {
    let syn_item: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &syn_item.ident;
    let variants = get_enum_variants(&syn_item);

    let variant_names = variants.iter().map(|v| {
        let mut name = v.ident.to_string().into_bytes();
        name.push(0);
        proc_macro2::Literal::byte_string(&name[..])
    });

    let expanded = quote! {
        impl InteropGetName for #name {
            fn interop_name(&self) -> &'static [u8] {
                const NAMES: &[&[u8]] = &[#(#variant_names),*];
                &NAMES[*self as usize]
            }
        }
    };
    expanded.into()
}

#[proc_macro_derive(EnumFromStr)]
pub fn enum_from_str_derive(input: TokenStream) -> TokenStream {
    let syn_item: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &syn_item.ident;
    let name_str = name.to_string();
    let variants = get_enum_variants(&syn_item);
    let literals = variants.iter().map(|v| v.ident.to_string());
    let identifiers = variants.iter().map(|v| &v.ident);

    let expanded = quote! {
        impl EnumFromStr for #name {
            fn from_str(s: &str) -> Result<#name, ::util::ParseEnumError> {
                match s {
                    #( #literals => Ok(#name::#identifiers), )*
                    _ => Err(::util::ParseEnumError {
                        value: s.to_string(),
                        enum_name: #name_str,
                    }),
                }
            }
        }

        impl ::core::str::FromStr for #name {
            type Err = ::util::ParseEnumError;

            fn from_str(s: &str) -> Result<#name, ::util::ParseEnumError> {
                EnumFromStr::from_str(s)
            }
        }
    };

    expanded.into()
}
