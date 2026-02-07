use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Path, parse_macro_input};

#[proc_macro_derive(Walker)]
pub fn derive_walker(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let walk_body = match &input.data {
        Data::Enum(data) => {
            let variants = data.variants.iter().map(|v| {
                let v_ident = &v.ident;
                match &v.fields {
                    // 1. Tuple-style variants: Variant(A, B)
                    // Inside the Enum variant matching logic of your proc_macro
                    Fields::Unnamed(fields) => {
                        // 1. Check how many fields are actually in this variant
                        let field_count = fields.unnamed.len(); 
                        
                        // 2. Generate a list of temporary names: _f0, _f1, _f2...
                        let vars: Vec<_> = (0..field_count)
                            .map(|i| quote::format_ident!("_f{}", i))
                            .collect();

                        quote! {
                            // 3. Use the "#( #vars ),*" syntax to expand them into a list
                            // This generates: Self::SumOfCardSet(_f0, _f1) => { ... }
                            Self::#v_ident( #( #vars ),* ) => {
                                #( #vars.walk(visitor); )*
                            }
                        }
                    }
                    // 2. Struct-style variants: Variant { a: T }
                    Fields::Named(fields) => {
                        let names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                        quote! {
                            Self::#v_ident { #( #names ),* } => {
                                #( #names.walk(visitor); )*
                            }
                        }
                    },
                    // 3. Unit variants: Variant (No fields like Eq, Neq)
                    Fields::Unit => quote! {
                        Self::#v_ident => {}
                    },
                }
            });
            quote! { match self { #( #variants )* } }
        }
        Data::Struct(data) => {
            match &data.fields {
                // 1. Named fields: struct Example { a: SIntExpr, b: SID }
                syn::Fields::Named(fields) => {
                    let f_names = fields.named.iter().map(|f| &f.ident);
                    quote! {
                        #( self.#f_names.walk(visitor); )*
                    }
                }
                // 2. Unnamed fields: struct Example(SIntExpr, SID)
                syn::Fields::Unnamed(fields) => {
                    // We use indices (self.0, self.1) for tuple structs
                    let indices = (0..fields.unnamed.len()).map(syn::Index::from);
                    quote! {
                        #( self.#indices.walk(visitor); )*
                    }
                }
                // 3. Unit structs: struct Example;
                syn::Fields::Unit => quote! {},
            }
        }
        _ => panic!("Unsupported type"),
    };

    quote! {
        impl Walker for #name {
            fn walk<V: AstPass>(&self, visitor: &mut V) {
                visitor.enter_node(self);
                #walk_body
                visitor.exit_node(self);
            }
            fn kind(&self) -> NodeKind<'_> {
                NodeKind::#name(self)
            }
        }
    }.into()
}

#[proc_macro_derive(Lower)] // Register the helper attribute
pub fn derive_lower(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let crate_path: syn::Path = syn::parse_str("crate::ast").expect("Failed to parse path");

    let walk_body = match &input.data {
        Data::Enum(data) => {
            let variants = data.variants.iter().map(|v| {
                let v_ident = &v.ident;
                match &v.fields {
                    Fields::Unnamed(fields) => {
                        let vars: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| quote::format_ident!("_f{}", i))
                            .collect();

                        quote! {
                            Self::#v_ident( #( #vars ),* ) => {
                                // Tuple variants use ()
                                #crate_path::#name::#v_ident( #( #vars.lower() ),* )
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                        quote! {
                            Self::#v_ident { #( #names ),* } => {
                                // Struct variants use { }
                                #crate_path::#name::#v_ident { #( #names: #names.lower() ),* }
                            }
                        }
                    },
                    Fields::Unit => quote! {
                        Self::#v_ident => #crate_path::#name::#v_ident,
                    },
                }
            });
            quote! { match self { #( #variants )* } }
        }
        Data::Struct(data) => {
            match &data.fields {
                syn::Fields::Named(fields) => {
                    let f_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                    quote! {
                        #crate_path::#name { #( #f_names: self.#f_names.lower() ),* }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    let indices = (0..fields.unnamed.len()).map(syn::Index::from);
                    quote! {
                        // Tuple structs use ()
                        #crate_path::#name( #( self.#indices.lower() ),* )
                    }
                }
                syn::Fields::Unit => quote! {
                    #crate_path::#name
                },
            }
        }
        _ => panic!("Only Structs and Enums are supported"),
    };

    quote! {
        impl Lower<#crate_path::#name> for #name {
            fn lower(&self) -> #crate_path::#name {
                #walk_body
            }
        }
    }.into()
}