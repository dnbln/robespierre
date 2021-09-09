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
            let config_ty = quote!{<#ty as ::robespierre::framework::standard::extractors::FromMessage>::Config};
            let config_ty_ufcs_root = quote! {<#config_ty as ::robespierre::framework::standard::extractors::ExtractorConfigBuilder>};
            let config_builder = {
                let tks = quote! {<<#ty as ::robespierre::framework::standard::extractors::FromMessage>::Config as std::default::Default>::default()};

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
                <#ty as ::robespierre::framework::standard::extractors::FromMessage>::from_message(ctx.clone(), __message.clone(), #config_builder).await?,
            });
        }
    }
}

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
