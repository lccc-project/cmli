use proc_macro::{Literal, TokenStream, TokenTree};
use proc_macro_deterministic_rand::{RandomSource, keys_from_cargo};
use proc_macro2::Span;

#[proc_macro]
pub fn rand_u64(st: TokenStream) -> TokenStream {
    let mut g = RandomSource::with_key_span_and_seed(Span::mixed_site(), keys_from_cargo!("cmli"));
    let mut ret = g.next(Span::mixed_site());
    for tt in st {
        ret = g.next(tt.span().into());
    }

    [TokenTree::Literal(Literal::u64_suffixed(ret))]
        .into_iter()
        .collect()
}
