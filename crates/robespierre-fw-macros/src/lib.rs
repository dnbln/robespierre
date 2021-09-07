use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg};

// Input:
//     async fn cmd(ctx: &FwContext, message: &Message, RawArgs(args): RawArgs) -> CommandResult {
//         // ...
//     }
// Output:
//     async fn __cmd_impl(ctx: &FwContext, message: &Message, RawArgs(args): RawArgs) -> CommandResult {
//         // ...
//     }
//     fn cmd<'a>(ctx: &'a ::robespierre::framework::standard::FwContext, message: &'a ::robespierre_models::channel::Message, args: &'a str) -> ::std::pin::Pin<::std::boxed::Box<dyn ::robespierre::futures::Future<Output = ::robespierre::framework::standard::CommandResult> + Send + 'a>>> {
//         ::std::boxed::Box::pin(async move {
//             let __message = ::robespierre::framework::standard::extractors::Msg {message: ::std::sync::Arc::new(message.clone()), args: ::std::sync::Arc::new(args.to_string())};
//             __cmd_impl(ctx, message, args, <RawArgs as ::robespierre::framework::standard::extractors::FromMessage>::from_message(ctx.clone(), __message.clone()).await?, ).await
//         })
//     }
#[proc_macro_attribute]
pub fn command(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut command_func = parse_macro_input!(item as syn::ItemFn);

    let impl_name = format_ident!("__{}_impl", &command_func.sig.ident);
    let old_name = std::mem::replace(&mut command_func.sig.ident, impl_name.clone());

    let visibility = std::mem::replace(&mut command_func.vis, syn::Visibility::Inherited);

    let extra_args = command_func
        .sig
        .inputs
        .iter()
        .skip(2)
        .filter_map(|it| match it {
            FnArg::Typed(a) => Some(&a.ty),
            FnArg::Receiver(_) => None,
        });

    let result = quote! {
        #command_func

        #visibility fn #old_name <'a> (
            ctx: &'a ::robespierre::framework::standard::FwContext,
            message: &'a ::robespierre_models::channel::Message,
            args: &'a str,
        ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::robespierre::framework::standard::CommandResult> + ::std::marker::Send + 'a>> {
            ::std::boxed::Box::pin(async move {
                let __message = ::robespierre::framework::standard::extractors::Msg {message: ::std::sync::Arc::new(message.clone()), args: ::std::sync::Arc::new(args.to_string())};
                #impl_name (
                    ctx,
                    message,
                    #(
                        <#extra_args as ::robespierre::framework::standard::extractors::FromMessage>::from_message(ctx.clone(), __message.clone()).await?,
                    )*
                ).await
            })
        }
    };
    proc_macro::TokenStream::from(result)
}
