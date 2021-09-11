use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, Expr, FnArg,
    Type,
};

#[allow(clippy::borrowed_box)]
struct ExtraArgs<'a>(&'a [&'a Box<Type>], &'a [Vec<Attribute>]);

impl<'a> ToTokens for ExtraArgs<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for (ty, attrs) in self.0.iter().zip(self.1.iter()) {
            let ty_from_message =
                quote! {<#ty as ::robespierre::framework::standard::extractors::FromMessage>};
            let config_ty = quote! {#ty_from_message ::Config};
            let config_ty_ufcs_root = quote! {<#config_ty as ::robespierre::framework::standard::extractors::ExtractorConfigBuilder>};
            let config_builder = {
                let tks = quote! {<#config_ty as ::std::default::Default>::default()};

                let tks = attrs.iter().fold(tks, |tks, attr| {
                    if attr.path.is_ident("delimiter") {
                        let expr = attr.parse_args::<Expr>().expect("parse as expr");
                        quote! { #config_ty_ufcs_root :: delimiter(#tks, #expr)}
                    } else if attr.path.is_ident("delimiters") {
                        struct DelimiterList(Punctuated<Expr, Comma>);

                        impl Parse for DelimiterList {
                            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                                Punctuated::<Expr, Comma>::parse_terminated(input).map(Self)
                            }
                        }

                        let delimiters = attr
                            .parse_args::<DelimiterList>()
                            .expect("parse as delimiter list")
                            .0;
                        quote! { #config_ty_ufcs_root :: delimiters(#tks, ::std::vec![#delimiters])}
                    } else {
                        panic!("Unknown attribute: {:?}", attr.path);
                    }
                });

                tks
            };
            tokens.extend(quote! {
                #ty_from_message ::from_message(ctx.clone(), __message.clone(), #config_builder).await?,
            });
        }
    }
}

// Input:
//     async fn cmd(ctx: &FwContext, message: &Message, RawArgs(args): RawArgs) -> CommandResult {
//         // ...
//     }
// Output:
//     fn cmd<'a>(ctx: &'a FwContext, message: &'a Arc<Message>, args: &'a str) -> Pin<Box<dyn Future<Output = CommandResult> + Send + 'a>>> {
//         async fn __cmd_impl(ctx: &FwContext, message: &Message, RawArgs(args): RawArgs) -> CommandResult {
//             // ...
//         }
//         Box::pin(async move {
//             let __message = Msg {message: Arc::clone(message), args: Arc::new(args.to_string())};
//             __cmd_impl(ctx, message, <RawArgs as FromMessage>::from_message(ctx.clone(), __message.clone()).await?, /* other extractors if present */).await
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

    let attrs = command_func
        .sig
        .inputs
        .iter_mut()
        .skip(2)
        .filter_map(|it| match it {
            FnArg::Typed(t) => {
                let (my_attrs, other_attrs): (Vec<_>, _) =
                    std::mem::take(&mut t.attrs).into_iter().partition(|attr| {
                        attr.path.is_ident("delimiter") || attr.path.is_ident("delimiters")
                    });
                t.attrs = other_attrs;

                Some(my_attrs)
            }
            FnArg::Receiver(_) => None,
        })
        .collect::<Vec<_>>();

    let extra_args = command_func
        .sig
        .inputs
        .iter()
        .skip(2)
        .filter_map(|it| match it {
            FnArg::Typed(a) => Some(&a.ty),
            FnArg::Receiver(_) => None,
        })
        .collect::<Vec<_>>();

    let extra_args = ExtraArgs(&extra_args, &attrs);

    let result = quote! {
        #visibility fn #old_name <'a> (
            ctx: &'a ::robespierre::framework::standard::FwContext,
            message: &'a ::std::sync::Arc<::robespierre_models::channel::Message>,
            args: &'a ::std::primitive::str,
        ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::robespierre::framework::standard::CommandResult> + ::std::marker::Send + 'a>> {
            #command_func

            ::std::boxed::Box::pin(async move {
                let __message = ::robespierre::framework::standard::extractors::Msg {message: ::std::sync::Arc::clone(message), args: ::std::sync::Arc::new(args.to_string())};
                #impl_name (
                    ctx,
                    message,
                    #extra_args
                ).await
            })
        }
    };
    proc_macro::TokenStream::from(result)
}
