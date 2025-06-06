use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn mini_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let block = &input.block;
    let attrs = &input.attrs;
    let vis = &input.vis;
    let result = quote! {
        #(#attrs)*
        #vis fn main() {
            // 直接运行 async 块
            // 必须在 main 函数体内声明 runtime
            let mut rt = ::mini_runtime::MiniRuntime::new();
            let fut = async #block;
            rt.spawn(fut);
            rt.run();
        }
    };
    result.into()
}