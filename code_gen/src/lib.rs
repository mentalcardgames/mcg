/*
    Manually writing repetitive logic for a walker or lowering-logic or even a mirrored spanned version of the original AST is very annoying.
    This crate allows to generate all of it.

    We can specify:
    #[spanned_ast] over a module and it will generate a sub-module with the Spanned-version, walker-logic and lowering-logic with it.

    If you have trouble with declaring the AST then change by your liking.
*/

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Fields, Ident, Item, ItemMod, Token, Variant, Visibility, parse_macro_input,
    punctuated::Punctuated,
};

// ===========================================================================
// ===========================================================================
// Main-Logic
// ===========================================================================
// ===========================================================================
#[proc_macro_attribute]
pub fn spanned_ast(_: TokenStream, item: TokenStream) -> TokenStream {
    let module = parse_macro_input!(item as ItemMod);

    let attrs = &module.attrs;
    let vis = &module.vis;
    let ident = &module.ident;

    let items = match &module.content {
        Some((_, items)) => items,
        None => {
            return syn::Error::new_spanned(
                &module,
                "spanned_ast requires an inline module { ... }",
            )
            .to_compile_error()
            .into();
        }
    };

    let spanned_mod = format_ident!("{}_spanned", ident);

    let mut spanned_items = Vec::new();
    let mut lower_impls = Vec::new();
    let mut type_items: Vec<Item> = Vec::new();

    for item in items {
        match item {
            Item::Struct(s) if matches!(s.vis, Visibility::Public(_)) => {
                let name = &s.ident;

                // Clone and rewrite fields
                let mut spanned_struct = s.clone();
                // 1. Scrub the Arbitrary derive
                scrub_arbitrary_derive(&mut spanned_struct.attrs);
                // 2. Span the fields
                span_fields(&mut spanned_struct.fields);
                spanned_items.push(Item::Struct(spanned_struct));

                // type SType = Spanned<Type>
                let alias = format_ident!("S{}", name);
                let type_item: Item = syn::parse_quote! {
                    pub type #alias = Spanned<#name>;
                };

                type_items.push(type_item);

                // Lower impl
                let lower_body = generate_struct_lower_spanned(name, &s.fields);

                lower_impls.push(quote! {
                    impl Lower<super::#name> for Spanned<#name> {
                        fn lower(&self) -> super::#name {
                            #lower_body
                        }
                    }
                });
            }

            Item::Enum(e) if matches!(e.vis, Visibility::Public(_)) => {
                let name = &e.ident;

                let mut spanned_enum = e.clone();
                scrub_arbitrary_derive(&mut spanned_enum.attrs);
                for variant in &mut spanned_enum.variants {
                    span_fields(&mut variant.fields);
                }

                spanned_items.push(Item::Enum(spanned_enum));

                let alias = format_ident!("S{}", name);
                let type_item: Item = syn::parse_quote! {
                    pub type #alias = Spanned<#name>;
                };

                type_items.push(type_item);

                let lower_body = generate_enum_lower_spanned(name, &e.variants);

                lower_impls.push(quote! {
                    impl Lower<super::#name> for Spanned<#name> {
                        fn lower(&self) -> super::#name {
                            #lower_body
                        }
                    }
                });
            }

            _ => {}
        }
    }

    // 1. Generate the variants
    let mut node_kinds = Vec::new();
    let mut walker_impls = Vec::new();
    // all items and adding the types
    let all_items = vec![spanned_items.clone(), type_items.clone()].concat();
    println!("Total items found in module: {}", items.len());

    for item in all_items.iter() {
        // Use a simpler visibility check to ensure we aren't skipping everything
        let item_vis = match item {
            Item::Struct(s) => &s.vis,
            Item::Enum(e) => &e.vis,
            Item::Type(t) => &t.vis,
            _ => continue,
        };

        // If it's not private, add it to the enum
        if matches!(item_vis, Visibility::Public(_)) {
            // Extract Ident and generate the specific walking logic for this item
            let (ident, walk_body) = match item {
                Item::Struct(s) => (&s.ident, generate_struct_walk(&s.fields)),
                Item::Enum(e) => (&e.ident, generate_enum_walk(&e.variants)),
                // Type aliases don't get Walker impls directly (usually the underlying type has it)
                // But we add them to the Enum so they can be "wrapped"
                Item::Type(t) => {
                    let id = &t.ident;
                    node_kinds.push(quote!(#id(&'a #id)));
                    continue;
                }
                _ => continue,
            };

            // Add to the Enum: VariantName(&'a TypeName)
            node_kinds.push(quote!(#ident(&'a #ident)));

            // Add the Walker Implementation for this specific struct/enum
            walker_impls.push(quote! {
                impl Walker for #ident {
                    fn walk<V: AstPass>(&self, visitor: &mut V) {
                        visitor.enter_node(self);
                        #walk_body
                        visitor.exit_node(self);
                    }
                    fn kind(&self) -> Option<NodeKind> {
                        Some(NodeKind::#ident(self))
                    }
                }
            });
        }
    }

    let output = quote! {
        #(#attrs)*
        #vis mod #ident {

            #(#items)*

            pub mod #spanned_mod {
                use super::*;
                use crate::{spans::*, lower::*, walker::*};

                #(#spanned_items)*

                #(#type_items)*

                #(#lower_impls)*

                #(#walker_impls)*

                pub enum NodeKind<'a> {
                    #(#node_kinds),*
                }
            }
        }
    };

    // Debug
    // println!("{}", output.to_string());

    output.into()
}

// ===========================================================================
// ===========================================================================
// Helper
// ===========================================================================
// ===========================================================================
fn span_fields(fields: &mut syn::Fields) {
    for field in fields.iter_mut() {
        field.ty = transform_type_spanned(&field.ty);

        // 2. Filter out attributes that shouldn't be in the spanned AST
        field.attrs.retain(|attr| {
            // Keep the attribute only if it's NOT 'arbitrary'
            !attr.path().is_ident("arbitrary") && !attr.path().is_ident("proptest")
        });
    }
}

fn is_spanned(type_path: &syn::TypePath) -> bool {
    type_path
        .path
        .segments
        .last()
        .map(|seg| seg.ident == "Spanned")
        .unwrap_or(false)
}

fn transform_type_spanned(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Path(type_path) => {
            // If already Spanned<...>, leave it alone
            if is_spanned(type_path) {
                return ty.clone();
            }

            let mut new_path = type_path.clone();
            let last_segment = new_path.path.segments.last_mut().unwrap();

            match &mut last_segment.arguments {
                syn::PathArguments::AngleBracketed(args) => {
                    // This is a container/wrapper
                    for arg in &mut args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            *inner_ty = transform_type_spanned(inner_ty);
                        }
                    }

                    // DO NOT WRAP OUTER TYPE
                    syn::Type::Path(new_path)
                }

                _ => {
                    // No generics → this is a leaf AST node
                    syn::parse_quote!(Spanned<#ty>)
                }
            }
        }

        syn::Type::Reference(reference) => {
            let mut new_ref = reference.clone();
            *new_ref.elem = *Box::new(transform_type_spanned(&reference.elem));
            syn::Type::Reference(new_ref)
        }

        syn::Type::Tuple(tuple) => {
            let mut new_tuple = tuple.clone();
            for elem in &mut new_tuple.elems {
                *elem = transform_type_spanned(elem);
            }
            syn::Type::Tuple(new_tuple)
        }

        _ => ty.clone(),
    }
}

// ===========================================================================
// ===========================================================================
// Lowering Logic
// ===========================================================================
// ===========================================================================
fn generate_struct_lower_spanned(name: &Ident, fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(fields) => {
            let names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();

            quote! {
                super::#name {
                    #( #names: self.node.#names.lower() ),*
                }
            }
        }

        Fields::Unnamed(fields) => {
            let indices = (0..fields.unnamed.len()).map(syn::Index::from);

            quote! {
                super::#name(
                    #( self.node.#indices.lower() ),*
                )
            }
        }

        Fields::Unit => {
            quote! { super::#name }
        }
    }
}

fn generate_enum_lower_spanned(
    name: &Ident,
    variants: &Punctuated<Variant, Token![,]>,
) -> proc_macro2::TokenStream {
    let arms = variants.iter().map(|v| {
        let v_ident = &v.ident;

        match &v.fields {
            Fields::Unnamed(fields) => {
                let vars: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| format_ident!("f{}", i))
                    .collect();

                quote! {
                    #name::#v_ident( #( #vars ),* ) => {
                        super::#name::#v_ident(
                            #( #vars.lower() ),*
                        )
                    }
                }
            }

            Fields::Named(fields) => {
                let names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();

                quote! {
                    #name::#v_ident { #( #names ),* } => {
                        super::#name::#v_ident {
                            #( #names: #names.lower() ),*
                        }
                    }
                }
            }

            Fields::Unit => {
                quote! {
                    #name::#v_ident => super::#name::#v_ident
                }
            }
        }
    });

    quote! {
        match &self.node {
            #(#arms),*
        }
    }
}

// ===========================================================================
// ===========================================================================
// Walking Logic
// ===========================================================================
// ===========================================================================
fn generate_struct_walk(fields: &syn::Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(f) => {
            let names = f.named.iter().map(|field| &field.ident);
            quote! { #( self.#names.walk(visitor); )* }
        }
        Fields::Unnamed(f) => {
            let indices = (0..f.unnamed.len()).map(syn::Index::from);
            quote! { #( self.#indices.walk(visitor); )* }
        }
        Fields::Unit => quote! {},
    }
}

fn generate_enum_walk(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
) -> proc_macro2::TokenStream {
    let arms = variants.iter().map(|v| {
        let v_ident = &v.ident;
        match &v.fields {
            Fields::Unnamed(fields) => {
                let vars: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| format_ident!("_f{}", i))
                    .collect();
                quote! { Self::#v_ident( #(#vars),* ) => { #(#vars.walk(visitor);)* } }
            }
            Fields::Named(fields) => {
                let names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                quote! { Self::#v_ident { #(#names),* } => { #(#names.walk(visitor);)* } }
            }
            Fields::Unit => quote! { Self::#v_ident => {} },
        }
    });
    quote! { match self { #(#arms)* } }
}

// ===========================================================================
// ===========================================================================
// Remove Traits that clash with the generated Spanned AST
// E.g. Arbitrary clashes and is not necessary to have for the Spanned AST
// (only the unspanned/lowered AST has trait Arbitrary!)
// ===========================================================================
// ===========================================================================
fn scrub_arbitrary_derive(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain_mut(|attr| {
        if attr.path().is_ident("derive") {
            if let Ok(nested) = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            ) {
                let filtered: Vec<_> = nested
                    .into_iter()
                    .filter(|path| !path.is_ident("Arbitrary"))
                    .collect();

                if filtered.is_empty() {
                    return false; // Drop the whole attribute
                }

                // Rewrite the attribute without Arbitrary
                let new_path = &attr.path();
                *attr = syn::parse_quote! { #[#new_path(#(#filtered),*)] };
                return true;
            }
        }

        // Drop any other arbitrary-specific helper attributes
        !attr.path().is_ident("arbitrary") && !attr.path().is_ident("proptest")
    });
}
