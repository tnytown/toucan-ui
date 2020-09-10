extern crate proc_macro;

use darling::{ast, util, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::*;

#[proc_macro_derive(TreeModel, attributes(root))]
pub fn derive_tree_model(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let err_ident: Ident = parse_quote!(INVALID);
    let root_ident = match &input.data {
        Data::Struct(x) => {
            let field = x.fields.iter().find(|field| {
                field
                    .attrs
                    .iter()
                    .find(|attr| **attr == parse_quote!(#[root]))
                    .is_some()
            });
            field
                .ok_or_else(|| format!("can't find root node annotation"))
                .and_then(|f| {
                    f.ident
                        .as_ref()
                        .ok_or(format!("we only support named structs"))
                })
                .unwrap_or_else(|err| {
                    input.ident.span().unwrap().error(err).emit();
                    &err_ident
                })
        }
        _ => {
            input.ident.span().unwrap().error("struct required").emit();
            &err_ident
        }
    };

    let im = quote! {(self as &dyn qmetaobject::QAbstractItemModel)};
    let model_impl = quote! {
        impl #impl_generics qmetaobject::QAbstractItemModel for #ident #ty_generics #where_clause {
            fn index(&self, r: i32, c: i32, parent: qmetaobject::QModelIndex) -> qmetaobject::QModelIndex {
                if !parent.is_valid() {
                    return #im.create_index(r, c, 0);
                }

                if let Some(ptr) = self.__pather_extend(parent.id(), parent.row()) {
                    return #im.create_index(r, c, ptr);
                }

                #im.create_index(-1, -1, 0)
            }

            fn parent(&self, index: qmetaobject::QModelIndex) -> qmetaobject::QModelIndex {
                if let Some((ptr, r)) = self.__pather_parent(index.id()) {
                    return #im.create_index(r, 0, ptr);
                }

                #im.create_index(-1, -1, 0)
            }

            fn row_count(&self, parent: qmetaobject::QModelIndex) -> i32 {
                self.#root_ident;
                0
            }

            fn column_count(&self, _parent: qmetaobject::QModelIndex) -> i32 {
                0
            }

            fn data(&self, index: qmetaobject::QModelIndex, role: i32) -> qmetaobject::QVariant {
                Default::default()
            }
        }
    };
    let pather_impl = impl_tree_pather(ident, &impl_generics, &ty_generics, where_clause);

    proc_macro::TokenStream::from(quote! {
        #model_impl

        #pather_impl
    })
}

fn impl_tree_pather(
    ident: &syn::Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: Option<&syn::WhereClause>,
) -> TokenStream {
    let dt = quote!(self.__pather_data.borrow());
    let dt_mut = quote!(self.__pather_data.borrow_mut());

    quote! {
        /// Implementation for tree path cache.
        /// Rust's ownership model doesn't jive with Qt, so we can't just hand pointers over to
        /// QModelIndex. We can store indices into our data instead, but the opaque 32 bit value
        /// isn't wide enough to store multiple indices for a path into a tree.
        impl #impl_generics #ident #ty_generics #where_clause {
            /// Get cached tree path sequence from a pointer.
            fn __pather_from_id(&self, id: usize) -> std::option::Option<impl std::ops::Deref<Target = [i32]> + '_> {
                if let Some(n) = #dt.get(id) {
                    // magic ids should only point to length values (negative).
                    if *n < 0 {
                        let sz = (-*n as usize);

                        // Ref::map is needed to extend the borrowed reference for a slice of the vector.
                        return Some(std::cell::Ref::map(#dt, |v| &v[id..id + sz]))
                    }
                }
                None
            }

            /// Duplicate an existing path sequence and add an element.
            fn __pather_extend(&self, id: usize, x: i32) -> std::option::Option<usize> {
                let orig = if let Some(orig) = self.__pather_from_id(id) {
                    (*orig).to_vec()
                } else { return None };
                let ptr = #dt.len();

                let mut v = #dt_mut;
                v.push(orig[0] - 1);
                v.extend_from_slice(&orig[1..]);
                v.push(x);

                Some(ptr)
            }

            /// Get parent path pointer and tip from child.
            fn __pather_parent(&self, id: usize) -> std::option::Option<(usize, i32)> {
                let v = #dt;

                // TODO(aptny): maybe fast-path for previous allocation given current length - 1
                // TODO(aptny): windows should be yielded in reverse order
                if let Some(n) = v.get(id) {
                    if *n >= 0 { return None; } // wtf?
                    let n = -*n as usize;
                    let mut pos = 0;

                    let cmp = &v[id + 1..id + n - 1]; // target path excluding length
                    for w in v[0..id].windows(n - 1) {
                        if w == cmp { return Some((pos, v[pos+n-1])); }
                        pos += 1;
                    }
                }

                None
            }
        }
    }
}
