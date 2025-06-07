use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn mini_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let block = &input.block;
    let attrs = &input.attrs;
    let vis = &input.vis;
    let result = quote! {
        #(#attrs)*
        #vis fn main() {
            let mut rt = ::mini_runtime::MiniRuntime::new();
            // 把 main 的 async 块也作为 future spawn
            ::mini_runtime::SPAWN_QUEUE.with(|queue| {
                queue.borrow_mut().push(Box::pin(async #block));
            });
            rt.run();
        }
    };
    result.into()
}
