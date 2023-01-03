#![doc = include_str!("./lib.md")]


use proc_macro::TokenStream;
use quote::quote;
use syn;



fn has_nested_flag_attribute(attr: &syn::Attribute, name: &'static str, flag: &'static str) -> bool {
    if let Ok(meta) = attr.parse_meta() {
        if let Some(ident) = meta.path().get_ident() {
            if &ident.to_string()==name {
                if let syn::Meta::List(list) = meta {
                    for nested in list.nested.iter() {
                        if let syn::NestedMeta::Meta(nmeta) = nested {
                            if let syn::Meta::Path(path) = nmeta {
                                let path = path.get_ident().expect("Invalid attribute syntax! (no ident)").to_string();
                                if &path==flag {
                                    return true
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

fn has_nested_flag_attribute_list(list: &Vec<syn::Attribute>, name: &'static str, flag: &'static str) -> bool {
    for attr in list.iter() {
        if has_nested_flag_attribute(attr, name, flag) {
            return true;
        }
    }

    false
}

fn extract_ignored_generics_list(list: &Vec<syn::Attribute>) -> Vec<String> {
    let mut collection = Vec::new();

    for attr in list.iter() {
        let mut list = extract_ignored_generics(attr);

        collection.append(&mut list);
    }

    collection
}

fn extract_ignored_generics(attr: &syn::Attribute) -> Vec<String> {
    let mut collection = Vec::new();

    if let Ok(meta) = attr.parse_meta() {
        if let Some(ident) = meta.path().get_ident() {
            if &ident.to_string()!="get_size" {
                return collection;
            }
            if let syn::Meta::List(list) = meta {
                for nested in list.nested.iter() {
                    if let syn::NestedMeta::Meta(nmeta) = nested {
                        let ident = nmeta.path().get_ident().expect("Invalid attribute syntax! (no iden)");
                        if &ident.to_string()!="ignore" {
                            panic!("Invalid attribute syntax! Unknown name {:?}", ident.to_string());
                        }

                        if let syn::Meta::List(list) = nmeta {
                            for nested in list.nested.iter() {
                                if let syn::NestedMeta::Meta(nmeta) = nested {
                                    if let syn::Meta::Path(path) = nmeta {
                                        let path = path.get_ident().expect("Invalid attribute syntax! (no ident)").to_string();
                                        collection.push(path);
                                    }
                                }
                            }
                        }

                    }
                }
            }
        }
    }

    collection
}

// Add a bound `T: GetSize` to every type parameter T, unless we ignore it.
fn add_trait_bounds(
    mut generics: syn::Generics,
    ignored: &Vec<String>,
) -> syn::Generics {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            let name = type_param.ident.to_string();
            let mut found = false;
            for ignored in ignored.iter() {
                if ignored==&name {
                    found = true;
                    break;
                }
            }
            if found {
                continue;
            }
            type_param.bounds.push(syn::parse_quote!(GetSize));
        }
    }
    generics
}



#[proc_macro_derive(GetSize, attributes(get_size))]
pub fn derive_get_size(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

     // The name of the sruct.
    let name = &ast.ident;

    // Extract all generics we shall ignore.
    let ignored = extract_ignored_generics_list(&ast.attrs);

    // Add a bound `T: GetSize` to every type parameter T.
    let generics = add_trait_bounds(ast.generics, &ignored);

    // Extract the generics of the struct/enum.
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Traverse the parsed data to generate the individual parts of the function.
    match ast.data {
        syn::Data::Enum(data_enum) => {
            if data_enum.variants.is_empty() {
                // Empty enums are easy to implement.
                let gen = quote! {
                    impl GetSize for #name {}
                };
                return gen.into()
            }

            let mut cmds = Vec::with_capacity(data_enum.variants.len());

            for variant in data_enum.variants.iter() {
                let ident = &variant.ident;

                match &variant.fields {
                    syn::Fields::Unnamed(unnamed_fields) => {
                        let num_fields = unnamed_fields.unnamed.len();

                        let mut field_idents = Vec::with_capacity(num_fields);
                        for i in 0..num_fields {
                            let field_ident = String::from("v")+&i.to_string();
                            let field_ident = syn::parse_str::<syn::Ident>(&field_ident).unwrap();

                            field_idents.push(field_ident);
                        }

                        let mut field_cmds = Vec::with_capacity(num_fields);

                        for (i, _field) in unnamed_fields.unnamed.iter().enumerate() {
                            let field_ident = String::from("v")+&i.to_string();
                            let field_ident = syn::parse_str::<syn::Ident>(&field_ident).unwrap();

                            field_cmds.push(quote! {
                                total += GetSize::get_heap_size(#field_ident);
                            })
                        }

                        cmds.push(quote! {
                            Self::#ident(#(#field_idents,)*) => {
                                let mut total = 0;

                                #(#field_cmds)*;

                                total
                            }
                        });
                    }
                    syn::Fields::Named(named_fields) => {
                        let num_fields = named_fields.named.len();

                        let mut field_idents = Vec::with_capacity(num_fields);

                        let mut field_cmds = Vec::with_capacity(num_fields);

                        for field in named_fields.named.iter() {
                            let field_ident = field.ident.as_ref().unwrap();

                            field_idents.push(field_ident);

                            field_cmds.push(quote! {
                                total += GetSize::get_heap_size(#field_ident);
                            })
                        }

                        cmds.push(quote! {
                            Self::#ident{#(#field_idents,)*} => {
                                let mut total = 0;

                                #(#field_cmds)*;

                                total
                            }
                        });
                    }
                    syn::Fields::Unit => {
                        cmds.push(quote! {
                            Self::#ident => 0,
                        });
                    }
                }
            }

            // Build the trait implementation
            let gen = quote! {
                impl #impl_generics GetSize for #name #ty_generics #where_clause {
                    fn get_heap_size(&self) -> usize {
                        match self {
                            #(#cmds)*
                        }
                    }
                }
            };
            return gen.into();
        }
        syn::Data::Union(_data_union) => panic!("Deriving GetSize for unions is currently not supported."),
        syn::Data::Struct(data_struct) => {
            if data_struct.fields.is_empty() {
                // Empty structs are easy to implement.
                let gen = quote! {
                    impl GetSize for #name {}
                };
                return gen.into();
            }

            let mut cmds = Vec::with_capacity(data_struct.fields.len());

            for field in data_struct.fields.iter() {

                // Check if the value should be ignored. If so skip it.
                if has_nested_flag_attribute_list(&field.attrs, "get_size", "ignore") {
                    continue;
                }

                let ident = field.ident.as_ref().unwrap();

                cmds.push(quote! {
                    total += GetSize::get_heap_size(&self.#ident);
                })
            }

            // Build the trait implementation
            let gen = quote! {
                impl #impl_generics GetSize for #name #ty_generics #where_clause {
                    fn get_heap_size(&self) -> usize {
                        let mut total = 0;

                        #(#cmds)*;

                        total
                    }
                }
            };
            return gen.into();
        },
    }
}
