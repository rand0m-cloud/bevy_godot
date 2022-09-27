use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::*;

#[proc_macro_derive(NodeTreeView, attributes(node))]
pub fn derive_node_tree_view(item: TokenStream) -> TokenStream {
    let view = parse_macro_input!(item as DeriveInput);

    let expanded = node_tree_view(view).unwrap_or_else(Error::into_compile_error);

    TokenStream::from(expanded)
}

fn node_tree_view(input: DeriveInput) -> Result<TokenStream2> {
    let item = &input.ident;
    let data_struct = match &input.data {
        Data::Struct(data_struct) => data_struct,
        _ => {
            return Err(Error::new_spanned(
                input,
                "NodeTreeView must be used on structs",
            ))
        }
    };

    if matches!(data_struct.fields, Fields::Unit) {
        return Err(Error::new_spanned(
            input,
            "NodeTreeView must be used on structs with fields",
        ));
    }

    let mut field_errors = vec![];
    let field_exprs = data_struct
        .fields
        .iter()
        .map(|field| match create_get_node_expr(field) {
            Ok(expr) => {
                if let Some(name) = &field.ident {
                    quote! { #name : #expr, }
                } else {
                    quote! { #expr, }
                }
            }
            Err(e) => {
                field_errors.push(e);
                TokenStream2::new()
            }
        })
        .collect::<TokenStream2>();

    if !field_errors.is_empty() {
        let mut error = field_errors[0].clone();
        error.extend(field_errors[1..].iter().cloned());

        return Err(error);
    }

    let self_expr = if matches!(data_struct.fields, Fields::Named(_)) {
        quote! { Self { #field_exprs } }
    } else {
        quote! { Self ( #field_exprs ) }
    };

    let node_tree_view = quote! { ::bevy_godot::prelude::NodeTreeView };
    let subclass = quote! { ::bevy_godot::prelude::godot_prelude::SubClass };
    let node = quote! { ::bevy_godot::prelude::Node };
    let tref = quote! { ::bevy_godot::prelude::TRef };

    let expanded = quote! {
       impl #node_tree_view for #item {
           fn from_node<T: #subclass<#node>>(node: #tref<T>) -> Self {
               let node = node.upcast::<#node>();
               #self_expr
           }
       }
    };

    Ok(expanded)
}

fn create_get_node_expr(field: &Field) -> Result<TokenStream2> {
    let node_path: LitStr = field
        .attrs
        .iter()
        .find_map(|attr| {
            (attr.path.segments[0].ident == "node").then_some(())?;
            attr.parse_args().ok()
        })
        .ok_or_else(|| {
            Error::new_spanned(field, "NodeTreeView: every field must have a #[node(..)]")
        })?;

    let optional = if field.ty == parse_quote!(ErasedGodotRef) {
        false
    } else if field.ty == parse_quote!(Option<ErasedGodotRef>) {
        true
    } else {
        return Err(Error::new_spanned(
            field,
            "NodeTreeView: fields must have the type of ErasedGodotRef or Option<ErasedGodotRef>",
        ));
    };

    let expr = if optional {
        quote! {
            node.has_node(#node_path)
                .then(|| unsafe {
                    ErasedGodotRef::new(node.get_node(#node_path).unwrap().assume_unique())
                })
        }
    } else {
        quote! {
            unsafe {
                ErasedGodotRef::new(node.get_node(#node_path).unwrap().assume_unique())
            }
        }
    };

    Ok(expr)
}
