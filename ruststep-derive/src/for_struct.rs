use inflector::Inflector;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use std::convert::*;

use super::*;

/// This must be same between codegens
fn table_arg() -> syn::Ident {
    syn::Ident::new("table", Span::call_site())
}

struct FieldEntries {
    attributes: Vec<syn::Ident>,
    holder_types: Vec<syn::Type>,
    into_owned: Vec<TokenStream2>,
}

impl FieldEntries {
    fn parse(st: &syn::DataStruct) -> Self {
        let table_arg = table_arg();

        let mut attributes = Vec::new();
        let mut holder_types = Vec::new();
        let mut into_owned = Vec::new();

        for field in &st.fields {
            let ident = field
                .ident
                .as_ref()
                .expect("Tuple struct case is not supported");
            attributes.push(ident.clone());

            let ft: FieldType = field.ty.clone().try_into().unwrap();

            if is_use_place_holder(&field.attrs) {
                match &ft {
                    FieldType::Path(_) => {
                        into_owned.push(quote! { #ident.into_owned(#table_arg)? });
                    }
                    FieldType::Optional(_) => {
                        into_owned.push(
                        quote! { #ident.map(|holder| holder.into_owned(#table_arg)).transpose()? },
                    );
                    }
                    FieldType::List(_) => unimplemented!(),
                }
                holder_types.push(ft.as_holder().as_place_holder().into());
            } else {
                into_owned.push(quote! { #ident });
                holder_types.push(ft.into());
            }
        }
        FieldEntries {
            attributes,
            holder_types,
            into_owned,
        }
    }
}

pub fn def_holder(ident: &syn::Ident, st: &syn::DataStruct) -> TokenStream2 {
    let holder_ident = as_holder_ident(ident);
    let FieldEntries {
        attributes,
        holder_types,
        ..
    } = FieldEntries::parse(&st);
    quote! {
        /// Auto-generated by `#[derive(Holder)]`
        #[derive(Debug, Clone, PartialEq)]
        pub struct #holder_ident {
            #(#attributes: #holder_types),*
        }
    }
}

pub fn impl_holder(ident: &syn::Ident, table: &TableAttr, st: &syn::DataStruct) -> TokenStream2 {
    let name = ident.to_string().to_screaming_snake_case();
    let holder_ident = as_holder_ident(ident);
    let FieldEntries {
        attributes,
        into_owned,
        ..
    } = FieldEntries::parse(st);
    let attr_len = attributes.len();
    let TableAttr { table, .. } = table;
    let table_arg = table_arg();
    let ruststep = ruststep_crate();

    quote! {
        #[automatically_derived]
        impl #ruststep::tables::Holder for #holder_ident {
            type Table = #table;
            type Owned = #ident;
            fn into_owned(self, #table_arg: &Self::Table) -> #ruststep::error::Result<Self::Owned> {
                let #holder_ident { #(#attributes),* } = self;
                Ok(#ident { #(#attributes: #into_owned),* })
            }
            fn name() -> &'static str {
                #name
            }
            fn attr_len() -> usize {
                #attr_len
            }
        }
    } // quote!
}

pub fn impl_entity_table(ident: &syn::Ident, table: &TableAttr) -> TokenStream2 {
    let TableAttr { table, field } = table;
    let holder_ident = as_holder_ident(ident);
    let ruststep = ruststep_crate();

    quote! {
        #[automatically_derived]
        impl #ruststep::tables::EntityTable<#holder_ident> for #table {
            fn get_owned(&self, entity_id: u64) -> #ruststep::error::Result<#ident> {
                #ruststep::tables::get_owned(self, &self.#field, entity_id)
            }
            fn owned_iter<'table>(&'table self) -> Box<dyn Iterator<Item = #ruststep::error::Result<#ident>> + 'table> {
                #ruststep::tables::owned_iter(self, &self.#field)
            }
        }
    }
}
