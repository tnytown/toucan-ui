#![feature(proc_macro_diagnostic)]
extern crate proc_macro;

use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::TokenStream;
use quote::quote;
use spanned::Spanned;
use syn::{parse_quote as pquote, *};

#[derive(Debug, FromField)]
#[darling(attributes(tm))]
struct TreeNodeField {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    children: bool,
    #[darling(default)]
    skip: bool,
}

#[derive(Debug, FromVariant)]
struct TreeNodeVariant {
    ident: Ident,
    fields: ast::Fields<TreeNodeField>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(tm))]
struct TreeNodeData {
    ident: Ident,
    data: ast::Data<TreeNodeVariant, TreeNodeField>,
    generics: syn::Generics,
    #[darling(default)]
    columns: ColumnList,

    // ¯\_(ツ)_/¯
    item: String,
}

impl TreeNodeField {
    fn ident(&self) -> Ident {
        let err = pquote!(__Unnamed_field);
        self.ident
            .as_ref()
            .unwrap_or_else(|| {
                self.ty.span().unwrap().error("expected named field").emit();
                &err
            })
            .clone()
    }
}

#[derive(Debug, Default)]
struct ColumnList(Vec<String>);
type DErr = darling::Error;
impl FromMeta for ColumnList {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let r: darling::Result<Vec<String>> = items
            // NestedMeta
            .iter()
            // -> Result<Lit>
            .map(|m| match m {
                NestedMeta::Lit(x) => Ok(x),
                _ => Err(DErr::unsupported_shape("nested meta")),
            })
            // -> Result<String>
            .map(|l| {
                l.and_then(|l| match l {
                    Lit::Str(x) => Ok(x.value()),
                    _ => Err(DErr::unexpected_lit_type(l)),
                })
            })
            // -> Result<Vec<String>>
            .collect();

        r.map(ColumnList)
    }
}

#[proc_macro_derive(TreeNode, attributes(tm))]
pub fn derive_tree_node(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let node: TreeNodeData = TreeNodeData::from_derive_input(&input).unwrap();

    let ident = &node.ident;
    let (g_impl, g_ty, g_where) = node.generics.split_for_impl();
    //let children_ident = node.data

    let path: syn::Path = ident.clone().into();
    let self_exp: syn::Expr = pquote!(self);
    let data_fn = {
        let nth_exp: syn::Expr = pquote!(n);

        let field_exp = match &node.data {
            ast::Data::Enum(e) => gen_map_enum_variants(&self_exp, &path, &e, |fields| {
                gen_map_struct_nth_field(&nth_exp, &fields, |field| {
                    let f_ident = field.ident();
                    pquote! {{
                        //println!("data={}", #f_ident);
                        qmetaobject::QString::from((&#f_ident as &dyn std::string::ToString).to_string()).into()
                    }}
                })
            }),
            ast::Data::Struct(s) => gen_map_struct_nth_field(&nth_exp, &s, |field| {
                let f_ident = field.ident();
                pquote! {{
                    //println!("data={}", &#self_exp.#f_ident);
                    qmetaobject::QString::from((&#self_exp.#f_ident as &dyn std::string::ToString).to_string()).into()
                }}
            }),
        };

        field_exp
    };

    let field_fn = {
        let nth_exp: syn::Expr = pquote!(n);

        let field_exp = match &node.data {
            ast::Data::Enum(e) => gen_map_enum_variants(&self_exp, &path, &e, |fields| {
                gen_map_struct_nth_field(&nth_exp, &fields, |field| {
                    let f_str = field.ident().to_string();
                    pquote!(qmetaobject::QString::from(#f_str).into())
                })
            }),
            ast::Data::Struct(s) => gen_map_struct_nth_field(&nth_exp, &s, |field| {
                let f_str = field.ident().to_string();
                pquote!(qmetaobject::QString::from(#f_str).into())
            }),
        };

        field_exp
    };

    let gen_child_expr = |is_mut| {
        let is_mut = if is_mut { quote!(mut) } else { quote!() };
        match &node.data {
            ast::Data::Enum(e) => {
                let mut iter = e
                    .iter()
                    .map(|v| (v, v.fields.iter().find(|x| x.children)))
                    .filter(|(_, b)| b.is_some())
                    .map(|(a, b)| (a, b.unwrap()));

                match iter.next() {
                    Some(target) => {
                        if let Some(x) = iter.next() {
                            x.0.ident
                                .span()
                                .unwrap()
                                .error("duplicate `#[children]` attribute")
                                .help("only 1 attribute is allowed in the type")
                                .emit();

                            return Default::default();
                        }

                        let variant_ident = &target.0.ident;
                        let field_ident: syn::Ident = target.1.ident();
                        Some(quote! {
                            match self {
                                #ident::#variant_ident{ref #is_mut #field_ident, ..} => Some(#field_ident),
                                _ => None
                            }
                        })
                    }
                    None => None,
                }
            }
            ast::Data::Struct(s) => {
                // TODO(aptny): check for duplicates like in the enum branch
                s.fields.iter().find(|x| x.children).map(|x| {
                    let id = x.ident();
                    quote!(Some(&#is_mut #self_exp.#id))
                })
            }
        }
        .unwrap_or_else(|| quote!(None))
    };

    let child_expr_mut = gen_child_expr(true);
    let child_expr = gen_child_expr(false);

    let for_ty: TokenStream = node.item.parse().unwrap();
    let columns = &node.columns.0;
    let label = ident.to_string();
    let tn_impl = quote! {
        impl #g_impl _common::TreeNode for #ident #g_ty #g_where {
            fn columns(&self) -> &[&'static str] {
                &[#(#columns),*]
            }

            fn get(&self, n: usize) -> std::option::Option<&dyn _common::TreeNode> {
                #child_expr.and_then(|v: &Vec<#for_ty>| v.get(n))
                .map(|o| (o as &dyn _common::TreeNode))
            }

            fn get_mut(&mut self, n: usize) -> std::option::Option<&mut dyn _common::TreeNode> {
                #child_expr_mut.and_then(|v: &mut Vec<#for_ty>| v.get_mut(n))
                .map(|o| (o as &mut dyn _common::TreeNode))
            }

            fn len(&self) -> usize {
                #child_expr.map(|v: &Vec<#for_ty>| v.len()).unwrap_or(0usize)
            }

            #[allow(unused_variables)]
            fn data(&self, n: i32) -> qmetaobject::QVariant {
                #data_fn
            }

            #[allow(unused_variables)]
            fn field(&self, n: i32) -> qmetaobject::QVariant {
                #field_fn
            }

            fn set_data(&mut self, n: i32, data: qmetaobject::QVariant) -> bool {
                false
            }

            fn label(&self) -> &'static str {
                #label
            }
        }
    };

    proc_macro::TokenStream::from(tn_impl)
}

fn gen_map_enum_variants<F>(
    on: &syn::Expr,
    enum_path: &syn::Path,
    variants: &Vec<TreeNodeVariant>,
    f: F,
) -> syn::Expr
where
    F: FnMut(&ast::Fields<TreeNodeField>) -> syn::Expr,
{
    // paths for the enum matches
    let paths = variants.iter().map::<Path, _>(|v| {
        let ident = &v.ident;
        pquote!(#enum_path :: #ident)
    });

    // patterns for the enum matches from the paths
    let pttns: Vec<Pat> = variants
        .iter()
        .map(|v| v.fields.as_ref().map(|f| f.ident()).fields)
        .zip(paths)
        .map(|(fs, pth)| pquote!(#pth { #(#fs),* }))
        .collect();

    // exprs
    let exprs = variants.iter().map(|v| &v.fields).map(f);

    pquote! {
        match #on {
            #( #pttns => #exprs ),*
        }
    }
}

fn gen_map_struct_nth_field<F>(
    nth: &syn::Expr,
    fields: &ast::Fields<TreeNodeField>,
    f: F,
) -> syn::Expr
where
    F: FnMut(&TreeNodeField) -> syn::Expr,
{
    let indices = 0i32..;
    // https://github.com/TedDriggs/darling/pull/87
    let fields: Vec<syn::Expr> = fields
        .iter()
        .filter(|f| !f.skip && !f.children)
        .map(f)
        .collect();

    pquote! {
        match #nth {
            #( #indices => #fields, )*
            _ => false.into()
        }
    }
}

// TODO(aptny): darling-ize
#[proc_macro_derive(TreeModel, attributes(root))]
pub fn derive_tree_model(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;
    let (g_impl, g_ty, g_where) = input.generics.split_for_impl();

    let err_ident: Ident = parse_quote!(__Unknown_root_field);
    let span = input.ident.span().unwrap();
    let root_ident = match &input.data {
        Data::Struct(x) => {
            let field = x.fields.iter().find(|field| {
                field
                    .attrs
                    .iter()
                    .find(|attr| **attr == parse_quote!(#[root]))
                    .is_some()
            });

            fn emit<T, F>(f: F) -> impl FnOnce() -> Option<T>
            where
                F: FnOnce() -> proc_macro::Diagnostic,
            {
                || {
                    f().emit();
                    None
                }
            }

            field
                .or_else(emit(|| {
                    span.error("can't find a root node attribute")
                        .help("try annotating a field with `#[root]`")
                }))
                .map(|x| x.ident.as_ref())
                .unwrap_or_else(emit(|| {
                    span.error("can't find the root node field")
                        .help("consider naming the fields of the struct")
                }))
                .unwrap_or(&err_ident)
        }
        _ => {
            span.error("struct required").emit();
            &err_ident
        }
    };

    let root_expr = quote! {(&self.#root_ident as &dyn common::TreeNode)};

    let im = quote!((self as &dyn qmetaobject::QAbstractItemModel));
    let model_impl = quote! {
        impl #g_impl qmetaobject::QAbstractItemModel for #ident #g_ty #g_where {
            fn index(&self, r: i32, c: i32, parent: qmetaobject::QModelIndex) -> qmetaobject::QModelIndex {
                // if the parent is the root, return a pristine index.
                if !parent.is_valid() {
                    return #im.create_index(r, c, 0);
                }

                // return an index rooted at the parent.
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
                let path = if let Some(x) = self.__pather_from_id(parent.id()) {
                    x
                } else { return 0 };

                let node = path.iter().fold(Some(#root_expr), |node, x| {
                    node.and_then(|node| node.get(*x as usize))
                });

                let node = if parent.is_valid() {
                    node.and_then(|n| n.get(parent.row() as usize))
                } else {
                    node
                };

                node.map(|n| n.len() as i32).map(|n| {
                    n
                }).unwrap_or(0)
            }

            fn column_count(&self, _parent: qmetaobject::QModelIndex) -> i32 {
                #root_expr.columns().len() as i32
            }

            fn role_names(&self) -> std::collections::HashMap<i32, qmetaobject::QByteArray> {
                let mut m = std::collections::HashMap::new();
                m.insert(common::ItemDataRole::NodeLabelRole as i32, qmetaobject::QString::from("nodeLabel").into());
                m.insert(common::ItemDataRole::FieldLabelRole as i32, qmetaobject::QString::from("fieldLabel").into());
                //m.insert(common::ItemDataRole::FieldCountRole, qmetaobject::QString::from("fieldCount").into());
                m
            }

            fn data(&self, index: qmetaobject::QModelIndex, role: i32) -> qmetaobject::QVariant {
                if !index.is_valid() {
                    return Default::default();
                }

                let path = if let Some(x) = self.__pather_from_id(index.id()) {
                    x
                } else { return Default::default(); };


                let node = path.iter().fold(Some(#root_expr), |node, x| {
                    node.and_then(|node| node.get(*x as usize))
                }).and_then(|n| n.get(index.row() as usize));

                let role = <common::ItemDataRole as num_traits::FromPrimitive>::from_i32(role).unwrap_or_default();
                match role {
                    common::ItemDataRole::EditRole |
                    common::ItemDataRole::DisplayRole => node.map(|n| n.data(index.column())),
                    common::ItemDataRole::FieldLabelRole => node.map(|n| n.field(index.column())),
                    _ => Default::default()
                }.unwrap_or_default()
            }
        }
    };
    let pather_impl = impl_tree_pather(ident, &g_impl, &g_ty, g_where);

    proc_macro::TokenStream::from(quote! {
        extern crate num_traits;
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
        /// isn't wide enough to store multiple indices for a path into a tree. This set of methods
        /// provides a facility to do that.
        impl #impl_generics #ident #ty_generics #where_clause {
            /// Get cached tree path sequence from a pointer.
            fn __pather_from_id(&self, id: usize) -> std::option::Option<impl std::ops::Deref<Target = [i32]> + '_> {
                self.__pather_from_id_raw(id, true)
            }

            fn __pather_from_id_raw(&self, id: usize, offs: bool)
                                    -> std::option::Option<impl std::ops::Deref<Target = [i32]> + '_> {
                let offs = if offs { 1 } else { 0 };
                if let Some(n) = #dt.get(id) {
                    // magic ids should only point to length values (negative).
                    if *n <= 0 {
                        let sz = ((*n).abs() as usize);

                        // Ref::map is needed to extend the borrowed reference for a slice of the vector.
                        return Some(std::cell::Ref::map(#dt, |v| &v[id + offs..id + sz + 1])) // TODO(aptny):
                    }
                }
                None
            }

            /// Duplicate an existing path sequence and add an element.
            fn __pather_extend(&self, id: usize, x: i32) -> std::option::Option<usize> {
                let mut orig = if let Some(orig) = self.__pather_from_id_raw(id, false) {
                    (*orig).to_vec()
                } else { return None };
                let ptr = #dt.len();

                orig[0] -= 1;
                orig.push(x);

                let start_pos = id + orig.len() - 1;
                for (i, w) in #dt[start_pos..].windows(orig.len()).enumerate() {
                    if w == &orig[..] { return Some(start_pos+i) }
                }

                let mut v = #dt_mut;
                v.extend_from_slice(&orig);

                Some(ptr)
            }

            /// Get parent path pointer and tip from child.
            fn __pather_parent(&self, id: usize) -> std::option::Option<(usize, i32)> {
                let v = #dt;

                // TODO(aptny): maybe fast-path for previous allocation given current length - 1
                if let Some(n) = v.get(id) {
                    //println!("__pather_parent: n={}", n);
                    if *n >= 0 { return None; } // wtf?
                    let n = (*n).abs() as usize;

                    if n == 1 { // parent is at the toplevel
                        return Some((0, v[id+1]));
                    }

                    let cmp = &v[id + 1..id + n]; // target path w/o length -- we compare that manually
                    let parent_len = n - 1;
                    let max = id - parent_len;
                    let tip = v[id+n];

                    // parent's length doesn't include the length element, so parent_len + 1
                    return (0..max).rev().map(|x| (x, &v[x..x + parent_len + 1]))
                        // find actual match by length element, path
                        .find(|(_, v)| v[0] == -(parent_len as i32) && &v[1..] == cmp)
                        // return parent pointer and index
                        .map(|(i, v)| (i, tip));
                }

                None
            }
        }
    }
}
