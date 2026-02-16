use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, Item, ItemMod, Visibility, parse_macro_input};

#[proc_macro_attribute]
pub fn ast(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut module = parse_macro_input!(item as ItemMod);
    let enum_name = format_ident!("NodeKind");

    let attrs = &module.attrs;
    let vis = &module.vis;
    let ident = &module.ident;

    let items = match &mut module.content {
        Some((_, items)) => items,
        None => {
            return syn::Error::new_spanned(&module, "node_kinds requires an inline module { ... }")
                .to_compile_error()
                .into();
        }
    };

    let module_lowered = format_ident!("{}_lowered", ident);

    // Unspan and lowering logic
    let mut unspanned_items = Vec::new();
    let mut lower_impls = Vec::new();

    for item in items.iter() {
        match item {
            Item::Struct(s) => {
                let mut unspanned_s = s.clone();
                unspan_fields(&mut unspanned_s.fields);
                
                let name = &s.ident;
                let lower_logic = generate_struct_lower(name, &s.fields, &module_lowered);
                
                unspanned_items.push(Item::Struct(unspanned_s));
                lower_impls.push(quote! {
                    impl Lower<#module_lowered::#name> for #name {
                        fn lower(&self) -> #module_lowered::#name { #lower_logic }
                    }
                });
            }
            Item::Enum(e) => {
                let mut unspanned_e = e.clone();
                for v in &mut unspanned_e.variants { unspan_fields(&mut v.fields); }
                
                let name = &e.ident;
                let lower_logic = generate_enum_lower(name, &e.variants, &module_lowered);
                
                unspanned_items.push(Item::Enum(unspanned_e));
                lower_impls.push(quote! {
                    impl Lower<#module_lowered::#name> for #name {
                        fn lower(&self) -> #module_lowered::#name { #lower_logic }
                    }
                });
            }
            _ => {}
        }
    }

    let unspan_and_lower = quote! {
        #(#lower_impls)*

        pub mod #module_lowered {
            use serde::{Serialize, Deserialize};

            pub type ID = String;

            #(#unspanned_items)*
        }
    };

    // 1. Generate the variants
    let mut variants = Vec::new();
    let mut walker_impls = Vec::new();
    println!("Total items found in module: {}", items.len());

    for item in items.iter() {
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
                    variants.push(quote!(#id(&'a #id)));
                    continue; 
                },
                _ => continue,
            };

            // Add to the Enum: VariantName(&'a TypeName)
            variants.push(quote!(#ident(&'a #ident)));

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

    // 2. Build the output explicitly
    // This ensures the module structure is exactly what the compiler expects
    let output = quote! {
        #(#attrs)*
        #vis mod #ident {
            #(#items)*

            #[derive(Debug, Clone)] // Removed Copy as types inside might not be Copy
            pub enum #enum_name<'a> {
                #(#variants),*
            }

            #(#walker_impls)*

            #unspan_and_lower
        }
    };

    // DEBUG: This will show you exactly what is happening in your terminal
    // during 'cargo check' or 'cargo build'
    // println!("--- MACRO OUTPUT ---\n{}\n-------------------", output.to_string());

    output.into()
}

fn unspan_fields(fields: &mut syn::Fields) {
    for field in fields.iter_mut() {
        field.ty = transform_type(&field.ty);
    }
}

fn transform_type(ty: &syn::Type) -> syn::Type {
    match ty {
        // Handle Paths: SID, Spanned<T>, Vec<T>, Box<T>
        syn::Type::Path(type_path) => {
            let mut new_path = type_path.clone();
            for segment in &mut new_path.path.segments {
                let ident_str = segment.ident.to_string();

                // 1. Peel the Spanned layer
                if ident_str == "Spanned" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            return transform_type(inner_ty);
                        }
                    }
                } 
                // 2. Handle S-prefix Aliases (SIntExpr -> IntExpr)
                else if ident_str.starts_with('S') && ident_str.chars().nth(1).map_or(false, |c| c.is_uppercase()) {
                    segment.ident = syn::Ident::new(&ident_str[1..], segment.ident.span());
                }

                // 3. Recurse into Generics (Vec<SIntExpr> -> Vec<IntExpr>)
                if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    for arg in &mut args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            *inner_ty = transform_type(inner_ty);
                        }
                    }
                }
            }
            syn::Type::Path(new_path)
        }
        // Handle Tuples: (SIntExpr, SIntExpr) -> (IntExpr, IntExpr)
        syn::Type::Tuple(type_tuple) => {
            let mut new_tuple = type_tuple.clone();
            for elem in &mut new_tuple.elems {
                *elem = transform_type(elem);
            }
            syn::Type::Tuple(new_tuple)
        }
        // Handle Pointers/Boxes: &SIntExpr or Box<SIntExpr>
        syn::Type::Reference(type_ref) => {
            let mut new_ref = type_ref.clone();
            *new_ref.elem = transform_type(&new_ref.elem);
            syn::Type::Reference(new_ref)
        }
        _ => ty.clone(),
    }
}

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

fn generate_enum_walk(variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>) -> proc_macro2::TokenStream {
    let arms = variants.iter().map(|v| {
        let v_ident = &v.ident;
        match &v.fields {
            Fields::Unnamed(fields) => {
                let vars: Vec<_> = (0..fields.unnamed.len()).map(|i| format_ident!("_f{}", i)).collect();
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

fn generate_struct_lower(
    name: &syn::Ident, 
    fields: &syn::Fields,
    target_mod: &syn::Ident
) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let f_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
            quote! {
                #target_mod::#name { 
                    #( #f_names: self.#f_names.lower() ),* }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let indices = (0..fields.unnamed.len()).map(syn::Index::from);
            quote! {
                #target_mod::#name( #( self.#indices.lower() ),* )
            }
        }
        syn::Fields::Unit => quote! {
            #target_mod::#name
        },
    }
}
fn generate_enum_lower(
    name: &syn::Ident, 
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
    target_mod: &syn::Ident
) -> proc_macro2::TokenStream {
    let arms = variants.iter().map(|v| {
        let v_ident = &v.ident;
        // We must access the fields of the specific variant 'v'
        match &v.fields {
            syn::Fields::Unnamed(fields) => {
                let vars: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| quote::format_ident!("_f{}", i))
                    .collect();

                quote! {
                    Self::#v_ident( #( #vars ),* ) => {
                        #target_mod::#name::#v_ident( #( #vars.lower() ),* )
                    }
                }
            }
            syn::Fields::Named(fields) => {
                let names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                quote! {
                    Self::#v_ident { #( #names ),* } => {
                        #target_mod::#name::#v_ident { 
                            #( #names: #names.lower() ),* }
                    }
                }
            },
            syn::Fields::Unit => quote! {
                Self::#v_ident => #target_mod::#name::#v_ident,
            },
        }
    });
            
    quote! { 
        match self { 
            #(#arms)* } 
    }
}