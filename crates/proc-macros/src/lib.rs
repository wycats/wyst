#![feature(proc_macro_def_site)]

use darling::{util::Flag, FromMeta};
use derive_syn_parse::Parse;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_str, AttributeArgs, DeriveInput, Expr, Lit, Meta, NestedMeta, Token,
    Type, Visibility, WhereClause,
};
use syn::{Generics, Ident};
use unicode_xid::UnicodeXID;

#[derive(Debug, FromMeta)]
struct WystDataArgs {
    #[darling(default)]
    copy: Flag,
    #[darling(default)]
    new: Flag,
}

impl WystDataArgs {
    fn with_copy(self) -> WystDataArgs {
        WystDataArgs {
            copy: Flag::present(),
            ..self
        }
    }
}

macro_rules! macro_try {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => return TokenStream::from(e.write_errors()),
        }
    };
}

macro_rules! parse_args {
    ($args:tt as $ty:tt) => {{
        let attr_args = parse_macro_input!($args as AttributeArgs);
        macro_try!($ty::from_list(&attr_args))
    }};
}

#[proc_macro_derive(Display)]
pub fn display(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let display_struct = match get_struct(&ast, "wyst_new") {
        Ok(v) => v,
        Err(s) => return s,
    };

    let impl_display = display_struct.impl_trait(quote! { Display });

    let fields = display_struct.fields.iter().map(|StructField { id, .. }| {
        quote! {
            write!(f, "{}", self.#id)?;
        }
    });

    let expanded = quote! {
        #impl_display {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                #(#fields)*

                Ok(())
            }
        }
    };
    TokenStream::from(expanded)
}

#[derive(Debug)]
struct WystDisplayArgs {
    format_string: String,
    rest: Vec<Expr>,
}

impl WystDisplayArgs {
    fn from_meta_list<'a>(
        list: &mut impl Iterator<Item = &'a NestedMeta>,
    ) -> darling::Result<Self> {
        let format_string = match list.next() {
            Some(item) => String::from_nested_meta(item)?,
            None => return Err(darling::Error::too_few_items(1)),
        };

        let rest = list.map(|item| match item {
            syn::NestedMeta::Lit(Lit::Str(s)) => parse_str(&s.value()).map_err(|_err| {
                darling::Error::unsupported_shape(
                    "arguments to wyst_display() must parse as expressions",
                )
            }),

            _ => {
                return Err(darling::Error::unsupported_shape(
                    "arguments to wyst_display() must be string literals",
                ))
            }
        });

        Ok(WystDisplayArgs {
            format_string,
            rest: rest.collect::<Result<Vec<Expr>, _>>()?,
        })
    }
}

impl FromMeta for WystDisplayArgs {
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        (match *item {
            Meta::Path(_) => Self::from_word(),
            Meta::List(ref value) => WystDisplayArgs::from_meta_list(&mut value.nested.iter()),
            // Self::from_list(
            //     &value
            //         .nested
            //         .iter()
            //         .cloned()
            //         .collect::<Vec<syn::NestedMeta>>()[..],
            // ),
            Meta::NameValue(ref value) => Self::from_value(&value.lit),
        })
        .map_err(|e| e.with_span(item))
    }
}

#[proc_macro_attribute]
pub fn wyst_display(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);

    let WystDisplayArgs {
        format_string,
        rest,
    } = match WystDisplayArgs::from_meta_list(&mut attr_args.iter()) {
        Ok(a) => a,
        Err(e) => return TokenStream::from(darling::Error::write_errors(e)),
    };

    let ast = parse_macro_input!(input as DeriveInput);

    let s = match get_struct(&ast, "wyst_display") {
        Ok(s) => s,
        Err(stream) => return stream,
    };

    let format = quote! { #format_string, #(#rest,)* };
    let impl_display = s.impl_trait(quote! { std::fmt::Display });

    let expanded = quote! {
        #ast

        #impl_display {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, #format)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn wyst_data(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_args!(args as WystDataArgs);

    // Parse the string representation
    let ast = parse_macro_input!(input as DeriveInput);

    data(ast, args)
}

#[proc_macro_attribute]
pub fn wyst_copy(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_args!(args as WystDataArgs);

    // Parse the string representation
    let ast = parse_macro_input!(input as DeriveInput);

    data(ast, args.with_copy())
}

#[proc_macro_derive(new)]
pub fn derive_new(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    new(&ast)
}

fn get_struct<'input>(ast: &'input DeriveInput, source: &str) -> Result<Struct, TokenStream> {
    let s = match &ast.data {
        syn::Data::Struct(s) => s,
        syn::Data::Enum(_) => {
            return Err(TokenStream::from(
                quote! { compile_error!("{} is not implemented for enums yet", #source) },
            ))
        }
        syn::Data::Union(_) => {
            return Err(TokenStream::from(
                quote! { compile_error!("{} is not implemented for unions yet", #source) },
            ))
        }
    };

    if s.fields.iter().any(|f| f.ident.is_none()) {
        return Err(TokenStream::from(
            quote! { compile_error!("{} doesn't work on tuple structs yet", #source) },
        ));
    }

    let fields = s
        .fields
        .iter()
        .enumerate()
        .map(|(pos, f)| {
            let id = f.ident.clone().unwrap_or(format_ident!("arg_{}", pos));
            let colon = f
                .colon_token
                .expect("Expected `:` token (mistaken assumption)");
            let ty = f.ty.clone();

            StructField { id, colon, ty }
        })
        .collect();

    let name = ast.ident.clone();
    // let vis = ast.vis.clone();

    let DeriveInput {
        attrs: _,
        vis,
        ident,
        generics,
        data: _,
    } = ast.clone();

    let Generics {
        lt_token,
        params,
        gt_token,
        where_clause,
    } = generics;

    let params = quote! { #lt_token #params #gt_token }.into();
    let implement = quote! { impl #params #ident #params #where_clause };

    Ok(Struct {
        name,
        vis,
        params,
        implement,
        where_clause,
        fields,
    })
}

#[derive(Debug, Clone)]
struct Struct {
    name: Ident,
    vis: Visibility,
    params: proc_macro2::TokenStream,
    where_clause: Option<WhereClause>,
    implement: proc_macro2::TokenStream,
    fields: Vec<StructField>,
}

impl Struct {
    fn impl_trait(&self, trait_name: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let Self {
            name,
            params,
            where_clause,
            ..
        } = self;
        quote! { impl #params #trait_name for #name #params #where_clause }
    }
}

#[derive(Debug, Clone)]
struct StructField {
    id: Ident,
    colon: Token![:],
    ty: Type,
}

fn new(ast: &DeriveInput) -> TokenStream {
    let Struct {
        fields,
        implement,
        vis,
        name,
        params,
        ..
    } = match get_struct(ast, "wyst_new") {
        Ok(v) => v,
        Err(s) => return s,
    };

    let field_args = fields.iter().map(|StructField { id, colon, ty }| {
        let ty = quote! { impl Into<#ty>, };

        quote! { #id #colon #ty }
    });

    let construct_fields = fields.iter().map(|StructField { id, colon, .. }| {
        quote! { #id #colon #id .into(), }
    });

    // let DeriveInput {
    //     attrs: _,
    //     vis,
    //     ident,
    //     generics,
    //     data: _,
    // } = ast.clone();

    // let Generics {
    //     lt_token,
    //     params,
    //     gt_token,
    //     where_clause,
    // } = generics;

    // let params = quote! { #lt_token #params #gt_token };

    TokenStream::from(quote! {
        #implement {
            #vis fn new(#(#field_args)*) -> #name #params {
                #name {
                    #(#construct_fields)*
                }
            }
        }
    })
}

fn data(ast: DeriveInput, args: WystDataArgs) -> TokenStream {
    let attrs = quote! {
        Debug, Clone, Hash, Eq, PartialEq
    };

    let DeriveInput {
        ident, generics, ..
    } = ast.clone();

    let Generics {
        lt_token,
        params,
        gt_token,
        where_clause,
    } = generics.clone();

    let generics = quote! {
        #lt_token #params #gt_token
    };

    let copy_impl = if args.copy.is_some() {
        quote! {
            impl #generics Copy for #ident #generics #where_clause {}
        }
    } else {
        quote! {}
    };

    let new_impl = if args.new.is_some() {
        proc_macro2::TokenStream::from(new(&ast))
    } else {
        proc_macro2::TokenStream::from(quote! {})
    };

    let expanded = quote! {
        #[derive(#attrs)]
        #ast

        #copy_impl
        #new_impl
    };

    expanded.into()
}

#[derive(Parse)]
struct UnitTest {
    desc: syn::LitStr,
    _comma: Token![,],
    _or1: Token![|],
    _or2: Token![|],
    block: syn::Block,
}

#[proc_macro]
pub fn unit_test(input: TokenStream) -> TokenStream {
    let UnitTest { desc, block, .. } = parse_macro_input!(input as UnitTest);

    let id_span = desc.span();
    let desc = desc.value();
    let mut chars = desc.chars();
    let mut id = String::new();

    if let Some(char) = chars.next() {
        if let Some(char) = normalize_char(char, true) {
            id.push(char);
        }
    }

    for char in chars {
        if let Some(char) = normalize_char(char, false) {
            id.push(char)
        }
    }

    let id = Ident::new(&id, id_span);
    let id = format_ident!("test_{}", id);

    let expanded = quote! {
        #[test]
        #[allow(non_snake_case)]
        fn #id() -> Result<(), Box<std::error::Error>>
        {
            println!("{}", concat!("test: ", stringify!(#desc)));
            #block;
            Ok(())
        }
    };

    TokenStream::from(expanded)
}

fn normalize_char(c: char, is_start: bool) -> Option<char> {
    match c {
        char if is_start && UnicodeXID::is_xid_start(char) => Some(char),
        char if !is_start && UnicodeXID::is_xid_continue(char) => Some(char),
        char if char.is_whitespace() || char == '#' || char == '&' => Some('_'),
        _ => None,
    }
}
