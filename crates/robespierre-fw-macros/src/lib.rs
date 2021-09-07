use quote::{format_ident, quote};
use syn::parse_macro_input;

// Input:
//     async fn cmd(ctx: &FwContext, message: &Message, args: &str) -> CommandResult {
//         // ...
//     }
// Output:
//     async fn __cmd_impl(ctx: &FwContext, message: &Message, args: &str) -> CommandResult {
//         // ...
//     }
//     fn cmd<'a>(ctx: &'a ::robespierre::framework::standard::FwContext, message: &'a ::robespierre_models::channel::Message, args: &'a str) -> ::std::pin::Pin<::std::boxed::Box<dyn ::robespierre::futures::Future<Output = ::robespierre::framework::standard::CommandResult> + Send + 'a>>> {
//         ::std::boxed::Box::pin(__cmd_impl(ctx, message, args))
//     }
#[proc_macro_attribute]
pub fn command(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut command_func = parse_macro_input!(item as syn::ItemFn);

    let impl_name = format_ident!("__{}_impl", &command_func.sig.ident);
    let old_name = std::mem::replace(&mut command_func.sig.ident, impl_name.clone());

    let result = quote! {
        #command_func

        pub fn #old_name <'a> (
            ctx: &'a ::robespierre::framework::standard::FwContext,
            message: &'a ::robespierre_models::channel::Message,
            args: &'a str,
        ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::robespierre::async_std::future::Future<Output = ::robespierre::framework::standard::CommandResult> + Send + 'a>> {
            ::std::boxed::Box::pin(#impl_name (ctx, message, args))
        }
    };
    proc_macro::TokenStream::from(result)
}
