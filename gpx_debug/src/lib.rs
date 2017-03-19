/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate quote;
extern crate syn;
extern crate proc_macro;

use proc_macro::TokenStream;


#[proc_macro_derive(XmlDebug)]
pub fn derive_bug(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    impl_xmldebug(&ast).parse().unwrap()
}


fn impl_xmldebug(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    let attrs = match &ast.body {
        &syn::Body::Struct(syn::VariantData::Struct(ref v)) => {
            v.iter().map(|field| {
                let name = &field.ident.clone().expect("Structure has unnamed fields");
                let ref path = match &field.ty {
                    &syn::Ty::Path(_, ref path) => path,
                    _ => panic!("Wrong object type")
                };
                let simple = quote! {
                    write!(f, "{}: {:?}, ", stringify!(#name), self.#name)?;
                };
                match path.segments.last().expect("Type is empty").ident.as_ref() {
                    "Option" => {
                        quote! {
                            match self.#name {
                                Some(ref a) => {
                                    write!(f, "{}: {:?}, ", stringify!(#name), a)?;
                                }
                                None => {}
                            }
                        }
                    }
                    "Vec" => {
                        quote! {
                            if !self.#name.is_empty() { #simple }
                        }
                    }
                    _ => { simple }
                }
            })
        }
        _ => {
            panic!("Only regular structs supported");
        }
    };
    
    let const_name = syn::Ident::new(format!("_IMPL_XMLDEBUG_FOR_{}", name));
    
    quote! {
        #[allow(non_upper_case_globals)]
        const #const_name : () = {
            use std::fmt;
            impl fmt::Debug for #name {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{} {{ ", stringify!(#name))?;
                    #( #attrs )*
                    write!(f, "}}")
                }
            }
        };
    }
}
