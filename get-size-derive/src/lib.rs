#![doc = include_str!("./lib.md")]


use proc_macro::TokenStream;
use quote::quote;
use syn;
use attribute_derive::Attribute;



#[derive(Attribute, Default, Debug)]
#[attribute(ident = get_size)]
struct StructFieldAttribute {
    #[attribute(conflicts = [size_fn, ignore])]
    size: Option<usize>,
    #[attribute(conflicts = [size, ignore])]
    size_fn: Option<syn::Ident>,
    #[attribute(conflicts = [size, size_fn])]
    ignore: bool,
}



fn extract_ignored_generics_list(list: &Vec<syn::Attribute>) -> Vec<syn::PathSegment> {
    let mut collection = Vec::new();

    for attr in list.iter() {
        let mut list = extract_ignored_generics(attr);

        collection.append(&mut list);
    }

    collection
}

fn extract_ignored_generics(attr: &syn::Attribute) -> Vec<syn::PathSegment> {
    let mut collection = Vec::new();

    // Skip all attributes which do not belong to us.
    if !attr.meta.path().is_ident("get_size") {
        return collection;
    }

    // Make sure it is a list.
    let list = attr.meta.require_list().unwrap();

    // Parse the nested meta.
    // #[get_size(ignore(A, B))]
    list.parse_nested_meta(|meta| {
        // We only parse the ignore attributes.
        if !meta.path.is_ident("ignore") {
            return Ok(()); // Just skip.
        }

        meta.parse_nested_meta(|meta| {
            for segment in meta.path.segments {
                collection.push(segment);
            }

            Ok(())
        })?;

        Ok(())
    }).unwrap();

    collection
}

// Add a bound `T: GetSize` to every type parameter T, unless we ignore it.
fn add_trait_bounds(
    mut generics: syn::Generics,
    ignored: &Vec<syn::PathSegment>,
) -> syn::Generics {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            let mut found = false;
            for ignored in ignored.iter() {
                if ignored.ident==type_param.ident {
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
                                let (total_add, tracker) = GetSize::get_heap_size_with_tracker(#field_ident, tracker);
                                total += total_add;
                            })
                        }

                        cmds.push(quote! {
                            Self::#ident(#(#field_idents,)*) => {
                                let mut total = 0;

                                #(#field_cmds)*;

                                (total, tracker)
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
                                let (total_add, tracker) = GetSize::get_heap_size_with_tracker(#field_ident, tracker);
                                total += total_add;
                            })
                        }

                        cmds.push(quote! {
                            Self::#ident{#(#field_idents,)*} => {
                                let mut total = 0;

                                #(#field_cmds)*;

                                (total, tracker)
                            }
                        });
                    }
                    syn::Fields::Unit => {
                        cmds.push(quote! {
                            Self::#ident => (0, tracker),
                        });
                    }
                }
            }

            // Build the trait implementation
            let gen = quote! {
                impl #impl_generics GetSize for #name #ty_generics #where_clause {
                    fn get_heap_size(&self) -> usize {
                        let tracker = get_size::StandardTracker::default();

                        let (total, _) = GetSize::get_heap_size_with_tracker(self, tracker);

                        total
                    }

                    fn get_heap_size_with_tracker<TRACKER: get_size::GetSizeTracker>(
                        &self,
                        tracker: TRACKER,
                    ) -> (usize, TRACKER) {
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

            let mut unidentified_fields_count = 0; // For newtypes

            for field in data_struct.fields.iter() {

                // Parse all relevant attributes.
                let attr = StructFieldAttribute::from_attributes(&field.attrs).unwrap();

                // NOTE There will be no attributes if this is a tuple struct.
                if let Some(size) = attr.size {
                    cmds.push(quote! {
                        total += #size;
                    });

                    continue;
                } else if let Some(size_fn) = attr.size_fn {
                    let ident = field.ident.as_ref().unwrap();

                    cmds.push(quote! {
                        total += #size_fn(&self.#ident);
                    });

                    continue;
                } else if attr.ignore {
                    continue;
                }

                if let Some(ident) = field.ident.as_ref() {
                    cmds.push(quote! {
                        let (total_add, tracker) = GetSize::get_heap_size_with_tracker(&self.#ident, tracker);
                        total += total_add;
                    });
                } else {
                    let current_index = syn::Index::from(unidentified_fields_count);
                    cmds.push(quote! {
                        let (total_add, tracker) = GetSize::get_heap_size_with_tracker(&self.#current_index, tracker);
                        total += total_add;
                    });

                    unidentified_fields_count += 1;
                }
            }

            // Build the trait implementation
            let gen = quote! {
                impl #impl_generics GetSize for #name #ty_generics #where_clause {
                    fn get_heap_size(&self) -> usize {
                        let tracker = get_size::StandardTracker::default();

                        let (total, _) = GetSize::get_heap_size_with_tracker(self, tracker);

                        total
                    }

                    fn get_heap_size_with_tracker<TRACKER: get_size::GetSizeTracker>(
                        &self,
                        tracker: TRACKER,
                    ) -> (usize, TRACKER) {
                        let mut total = 0;

                        #(#cmds)*;

                        (total, tracker)
                    }
                }
            };
            return gen.into();
        },
    }
}
