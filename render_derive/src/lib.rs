#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
extern crate quote;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};


#[proc_macro_derive(VertexAttribPointers, attributes(location))]
pub fn vertex_attrib_pointers_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let where_clause = &input.generics.where_clause;
    let fields_vertex_attrib_pointer = generate_vertex_attrib_pointer_calls(&input.data);

    let gen = quote!{
        impl #name #generics #where_clause {
            #[allow(unused_variables)]
            pub fn vertex_attrib_pointers(gl: &::gl::Gl) {
                let stride = ::std::mem::size_of::<Self>();
                let offset = 0;

                #(#fields_vertex_attrib_pointer)*
            }
        }
    };
    proc_macro::TokenStream::from(gen)
}

fn generate_vertex_attrib_pointer_calls(body: &syn::Data) -> Vec<TokenStream> {
    match body {
        &syn::Data::Enum(_) => panic!("VertexAttribPointers can not be implemented for enums"),
        &syn::Data::Union(_) => panic!("VertexAttribPointers can not be implemented for a Union"),
        &syn::Data::Struct(ref s) => {
            s.fields.iter().map(generate_struct_field_vertex_attrib_pointer_call).collect()
        },
    
    }
}

fn generate_struct_field_vertex_attrib_pointer_call(field: &syn::Field) -> TokenStream {
    let field_name = match field.ident {
        Some(ref i) => format!("{}", i),
        None => String::from(""),
    };
    let location_attr = field.attrs
        .iter()
        .filter(|a| a.path.is_ident("location"))
        .next()
        .unwrap_or_else(|| panic!(
            "Field {} is missing #[location = ?] attribute", field_name
        ));

    let vals = location_attr.parse_meta().unwrap();
    let location_value: usize = match vals {
        syn::Meta::NameValue(syn::MetaNameValue { ref ident, ref lit, .. }) if ident == "location" => (
                if let syn::Lit::Str(lit) = lit {
                    usize::from_str_radix(&lit.value(), 10).unwrap()
                }
                else if let syn::Lit::Int(lit) = lit {
                    lit.value() as usize
                }
                else {
                    panic!("Field {} location attribute value is not parseable", field_name)
                }
            ),
        _ => panic!("Field {} location attribute value must be a string literal", field_name)
    };

    let field_ty = &field.ty;
    quote! {
        let location = #location_value;
        unsafe {
            #field_ty::vertex_attrib_pointer(gl, stride, location, offset);
        }
        let offset = offset + ::std::mem::size_of::<#field_ty>();
    }
}