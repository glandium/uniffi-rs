/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
#![cfg_attr(feature = "nightly", feature(proc_macro_expand))]

//! Custom derive for uniffi_meta::Checksum

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Index};

#[proc_macro_derive(Checksum)]
pub fn checksum_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let code = match input.data {
        Data::Enum(enum_)
            if enum_.variants.len() == 1
                && enum_
                    .variants
                    .iter()
                    .all(|variant| matches!(variant.fields, Fields::Unit)) =>
        {
            quote!()
        }
        Data::Enum(enum_) => {
            let match_inner = enum_.variants.iter().enumerate().map(|(num, variant)| {
                let num = num as u64;
                let ident = &variant.ident;
                if variant.discriminant.is_some() {
                    panic!("#[derive(Checksum)] doesn't support explicit discriminants in enums");
                }
                let discriminant = quote! { state.write(&#num.to_le_bytes()) };
                match &variant.fields {
                    Fields::Unnamed(fields) => {
                        let field_idents = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(num, _)| format_ident!("__self_{}", num))
                            .collect::<Vec<_>>();
                        let field_stmts = field_idents
                            .iter()
                            .map(|ident| quote! { Checksum::checksum(#ident, state); });
                        quote! {
                            Self::#ident(#(#field_idents,)*) => {
                                #discriminant;
                                #(#field_stmts)*
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let field_idents = fields
                            .named
                            .iter()
                            .map(|field| field.ident.as_ref().unwrap())
                            .collect::<Vec<_>>();
                        let field_stmts = field_idents
                            .iter()
                            .map(|ident| quote! { Checksum::checksum(#ident, state); });
                        quote! {
                            Self::#ident { #(#field_idents,)* } => {
                                #discriminant;
                                #(#field_stmts)*
                            }
                        }
                    }
                    Fields::Unit => quote! { Self::#ident => #discriminant, },
                }
            });
            quote! {
                match self {
                    #(#match_inner)*
                }
            }
        }
        Data::Struct(struct_) => {
            let stmts =
                struct_
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(num, field)| match field.ident.as_ref() {
                        Some(ident) => quote! { Checksum::checksum(&self.#ident, state); },
                        None => {
                            let i = Index::from(num);
                            quote! { Checksum::checksum(&self.#i, state); }
                        }
                    });
            quote! {
                #(#stmts)*
            }
        }
        Data::Union(_) => {
            panic!("#[derive(Checksum)] is not supported for unions");
        }
    };

    quote! {
        impl #impl_generics Checksum for #name #ty_generics #where_clause {
            fn checksum<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
                #code
            }
        }
    }
    .into()
}
