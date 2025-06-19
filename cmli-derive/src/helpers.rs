use std::{
    hash::{Hash, Hasher},
    iter::FusedIterator,
};

use itertools::Itertools;
use lccc_siphash::SipHasher;
use proc_macro::{Ident, Literal, Punct, Span, TokenStream, TokenTree};

pub fn hash_span<H: Hasher>(span: Span, hasher: &mut H) {
    span.byte_range().hash(hasher);
    span.file().hash(hasher);
}

pub fn hash_token_tree<H: Hasher>(tt: TokenTree, hasher: &mut H) {
    hash_span(tt.span(), hasher);
    match tt {
        TokenTree::Group(group) => {
            core::mem::discriminant(&group.delimiter()).hash(hasher);
            hash_token_stream(group.stream(), hasher);
        }
        TokenTree::Ident(ident) => ident.to_string().hash(hasher),
        TokenTree::Punct(punct) => {
            punct.as_char().hash(hasher);
            core::mem::discriminant(&punct.spacing()).hash(hasher);
        }
        TokenTree::Literal(literal) => literal.to_string().hash(hasher),
    }
}

pub fn hash_token_stream<H: Hasher>(ts: TokenStream, hasher: &mut H) {
    ts.into_iter().for_each(|tt| hash_token_tree(tt, hasher));
    hasher.write_usize(0xDEADBEEF);
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

const HASH_SEED_VERSION: &str = env!("HASH_SEED_VERSION");
const HASH_KEY_TARGET: &str = env!("TARGET");

pub fn seed_token_generator() -> SipHasher<2, 4> {
    let mut seed_gen = SipHasher::<2, 4>::new_with_keys(11717105939243852261, 11816760824499105823);
    VERSION.hash(&mut seed_gen);
    NAME.hash(&mut seed_gen);
    HASH_SEED_VERSION.hash(&mut seed_gen);
    let k0 = seed_gen.finish();
    HASH_KEY_TARGET.hash(&mut seed_gen);
    let k1 = seed_gen.finish();

    SipHasher::new_with_keys(k0, k1)
}

struct Path<I: Iterator, S: Iterator> {
    inner: I,
    lookahead: Option<I::Item>,
    sep: S,
    cur_sep: Option<S>,
}

impl<I: Iterator, S: Iterator> Path<I, S> {
    pub fn new(mut iter: I, sep: S) -> Self {
        let first = iter.next();
        Self {
            inner: iter,
            lookahead: first,
            sep,
            cur_sep: None,
        }
    }
}

impl<I: FusedIterator, S: Iterator<Item = I::Item>> Iterator for Path<I, S>
where
    S: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur_sep) = self
            .lookahead
            .as_ref()
            .and_then(|_| self.cur_sep.as_mut())
            .and_then(Iterator::next)
        {
            return Some(cur_sep);
        }

        let next = self.lookahead.take();
        self.lookahead = self.inner.next();
        self.cur_sep = self.lookahead.as_ref().map(|_| self.sep.clone());

        next
    }
}

pub fn path<'a, I: IntoIterator<Item = &'a str>>(
    p: I,
    span: Span,
) -> impl Iterator<Item = TokenTree> + 'a
where
    I::IntoIter: 'a + FusedIterator,
{
    Path::new(
        p.into_iter()
            .map(move |id| TokenTree::Ident(Ident::new(id, span))),
        punct("::", span),
    )
}
pub fn punct(punct: &str, span: Span) -> impl Iterator<Item = TokenTree> + Clone + '_ {
    let mut chars = punct.chars();
    let last = chars.next_back();

    chars
        .map(|c| Punct::new(c, proc_macro::Spacing::Joint))
        .chain(last.map(|c| Punct::new(c, proc_macro::Spacing::Alone)))
        .update(move |p| p.set_span(span))
        .map(TokenTree::Punct)
}

pub fn lit_u64(n: u64) -> impl Iterator<Item = TokenTree> {
    [TokenTree::Literal(Literal::u64_suffixed(n))].into_iter()
}

pub fn next_ident<I: Iterator<Item = proc_macro2::TokenTree>>(
    it: &mut I,
) -> Option<proc_macro2::Ident> {
    for tt in it {
        match tt {
            proc_macro2::TokenTree::Ident(id) => return Some(id),
            _ => continue,
        }
    }
    None
}
