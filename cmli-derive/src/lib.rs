#![feature(
    proc_macro_span,
    proc_macro_diagnostic,
    array_into_iter_constructors,
    proc_macro_quote,
    proc_macro_expand
)]

use std::hash::{Hash, Hasher};

use helpers::{lit_u64, path};
use proc_macro::{Delimiter, Span, TokenStream, TokenTree};

use crate::helpers::{hash_token_stream, next_ident};

mod helpers;

fn derive_impl<const N: usize>(item: TokenStream, loc: [&str; N], tr: &str) -> TokenStream {
    let pkg = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| String::from("blob"));
    let mut key_gen = helpers::seed_token_generator();
    tr.hash(&mut key_gen);
    pkg.hash(&mut key_gen);

    let mut iter = item.into_iter().peekable();

    while let Some(tt) = iter.peek() {
        match tt {
            TokenTree::Punct(p) if p.as_char() == '#' => {
                iter.next();
                // attribute
                match iter.next().expect("There should be a token") {
                    TokenTree::Group(_) => {
                        // TODO: consume group
                    }
                    _ => panic!("There should be a group after `#`"),
                }
            }
            _ => break,
        }
    }

    match iter.peek().expect("There should be a token") {
        TokenTree::Ident(id) if id.to_string() == "pub" => {
            iter.next();

            match iter.peek().expect("There should be a token") {
                TokenTree::Group(_) => drop(iter.next()),
                _ => {}
            }
        }
        _ => {}
    }

    let is_enum = match iter.next().expect("Expected `enum` or `struct`") {
        TokenTree::Ident(id) if id.to_string() == "enum" => true,
        TokenTree::Ident(id) if id.to_string() == "struct" => false,
        tt => {
            tt.span()
                .error(format!("Expected a `enum`, got {tt}"))
                .help(format!("`#[derive({tr})]` only works for `enum` types"))
                .emit();
            return TokenStream::new();
        }
    };

    let name: proc_macro2::TokenStream = match iter.next().expect("Expected a name") {
        TokenTree::Ident(id) => {
            let stream: TokenStream = quote::quote!(::core::module_path!()).into();
            let stream = stream
                .expand_expr()
                .expect("Expected to expand module_path!()");
            hash_token_stream(stream, &mut key_gen);
            helpers::hash_span(id.span(), &mut key_gen);
            id.to_string().hash(&mut key_gen);
            [TokenTree::Ident(id)]
                .into_iter()
                .collect::<TokenStream>()
                .into()
        }
        _ => panic!("Expected a name"),
    };

    let mut val = key_gen.finish();

    while val == 0 {
        key_gen.update(0x6a09e667f3bcc908);
        val = key_gen.finish();
    }

    let id: proc_macro2::TokenStream = lit_u64(val).collect::<TokenStream>().into();

    let cs = Span::call_site();

    let ty: proc_macro2::TokenStream = path(
        core::iter::once("cmli")
            .chain(loc)
            .chain(core::iter::once(tr)),
        cs,
    )
    .collect::<TokenStream>()
    .into();

    if is_enum {
        let inner: proc_macro2::TokenStream = match iter.next().expect("Expected a body") {
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => g.stream(),
            tt => panic!("Expected a braced enum body, got {tt:?}"),
        }
        .into();

        let mut downcast_arms = proc_macro2::TokenStream::new();
        let mut into_id_arms = proc_macro2::TokenStream::new();

        let mut discrim = 0u64;

        let mut iter = inner.into_iter();
        while let Some(id) = next_ident(&mut iter) {
            into_id_arms.extend(quote::quote! { Self::#id => #discrim,});
            downcast_arms
                .extend(quote::quote! { #discrim => ::core::option::Option::Some(Self::#id),});
            discrim += 1;
        }

        quote::quote! {
            impl cmli::traits::IntoId<#ty> for #name {
                const __ID_TYPE: ::core::num::NonZeroU64 = unsafe{ ::core::num::NonZeroU64::new_unchecked(#id) };
                fn __into_id(self) -> u64 {
                    match self {
                        #into_id_arms
                    }
                }

                fn __try_downcast(val: u64) -> ::core::option::Option<Self> {
                    match val {
                        #downcast_arms
                        _ => ::core::option::Option::None
                    }
                }
            }
        }
        .into()
    } else {
        quote::quote! {
            impl cmli::traits::IntoId<#ty> for #name {
                const __ID_TYPE: u64 = #id;
                fn into_id(self) -> #ty {
                    let Self(val) = self;
                    let val: impl cmli::traits::AsU64 = val;

                    <#ty>::__new_unchecked(Self::__ID_TYPE, unsafe { core::mem::transmute(val) })
                }

                fn try_downcast(val: u64) -> Option<Self> {
                    Some(Self(cmli::traits::AsU64::__value_from(val)?;))
                }
            }
        }
        .into()
    }
}

macro_rules! derive_traits {
    ($($({$($path:ident)::+} ::)? $tr:ident),* $(,)?) => {
        $(#[proc_macro_derive($tr, attributes(cmli))]
        #[allow(nonstandard_style)]
        pub fn $tr(item: TokenStream) -> TokenStream {
            derive_impl(
                item,
                [$($(::core::stringify!($path)),*)?],
                ::core::stringify!($tr),
            )
        })*
    };
}

derive_traits! {
    {instr}:: InstructionId,
    {instr}:: InstructionFieldId,
    {instr}:: EncodingId,
    {instr}:: RegisterId,
    {instr}:: RegisterClassId,
    {instr}:: ExtendedConditionId,
    {instr}:: PrefixId,
    {instr}:: RelocTypeId,
    {encode}:: RelocId,
    {arch}:: MachId,
    {arch}:: FeatureId,
}

#[proc_macro]
pub fn compile_error_with_span(ts: TokenStream) -> TokenStream {
    let ts: proc_macro2::TokenStream = ts.into();
    let mut input = ts.into_iter();

    let Some(span_source) = input.next() else {
        panic!("Must have a valid span token")
    };

    match input.next() {
        Some(proc_macro2::TokenTree::Punct(c)) if c.as_char() == ',' => (),
        Some(tt) => panic!("Expected `,` not {tt}"),
        None => (),
    }

    let span: proc_macro2::Span = span_source.span();
    let rest = input.collect::<proc_macro2::TokenStream>();
    quote::quote_spanned! {span => ::core::compile_error!(#rest);}.into()
}
