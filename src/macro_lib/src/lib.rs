use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

// 使用 `proc_macro_attribute` 属性声明这是一个属性宏
#[proc_macro_attribute]
pub fn log_duration(_args: TokenStream, input: TokenStream) -> TokenStream {
    // 1. 解析输入：将原始的 TokenStream 解析为函数项的语法树
    let input_fn = parse_macro_input!(input as ItemFn);

    // 2. 提取函数的各个组成部分
    let vis = &input_fn.vis;           // 可见性 (pub, pub(crate), 等)
    let sig = &input_fn.sig;           // 函数签名 (fn name(args) -> ReturnType)
    let attrs = &input_fn.attrs;       // 属性 (如 #[inline])
    let function_name = &input_fn.sig.ident; // 获取函数名
    let function_block = &input_fn.block; // 获取原始函数体

    // 3. 生成新代码：使用 quote! 宏模板生成新的代码
    let expanded = quote! {
        // 保留原函数的属性、可见性和签名
        #(#attrs)*
        #vis #sig {
            // 在函数体开始前插入代码：记录开始时间并打印日志
            let start = std::time::Instant::now();
            println!("▶️ 函数 `{}` 开始执行", stringify!(#function_name));

            // 执行原始函数体，并将结果存储在 `__result` 变量中
            let __result = (|| #function_block)();

            // 在函数体结束后插入代码：计算耗时并打印结果
            let duration = start.elapsed();
            println!("⏹️ 函数 `{}` 执行完毕，耗时: {:?}", stringify!(#function_name), duration);

            // 返回原始函数的执行结果
            __result
        }
    };

    // 3. 返回结果：将生成的代码转换回 TokenStream 返回给编译器
    TokenStream::from(expanded)
}
