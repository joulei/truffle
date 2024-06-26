use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_macro_input, Expr, ExprPath, ItemFn, Token};

struct RegisterFnInput {
    engine: Expr,
    name: Expr,
    fun: ExprPath,
}

impl Parse for RegisterFnInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let engine = input.parse()?;
        input.parse::<Token![,]>()?;
        let name = input.parse()?;
        input.parse::<Token![,]>()?;
        let fun = input.parse()?;
        Ok(RegisterFnInput { engine, name, fun })
    }
}

#[proc_macro]
pub fn register_fn(input: TokenStream) -> TokenStream {
    let RegisterFnInput { engine, name, fun } = parse_macro_input!(input);
    let fun_is_async = {
        let mut path = fun.path.clone();
        let ident_segment = path
            .segments
            .last_mut()
            .expect("path should always have at least one segment");
        ident_segment.ident = format_ident!("{}_is_async", ident_segment.ident);
        path
    };
    let register_fun = {
        let mut path = fun.path.clone();
        let ident_segment = path
            .segments
            .last_mut()
            .expect("path should always have at least one segment");
        ident_segment.ident = format_ident!("register_{}", ident_segment.ident);
        path
    };
    let fn_location = {
        let mut path = fun.path.clone();
        let ident_segment = path
            .segments
            .last_mut()
            .expect("path should always have at least one segment");
        ident_segment.ident = format_ident!("{}_location", ident_segment.ident);
        path
    };

    #[cfg(feature = "lsp")]
    {
        quote! {
            if #fun_is_async() {
                #engine.with(#register_fun(#name))
            } else {
                #engine.register_fn(#name, #fun, Some(#fn_location()))
            };
            if #engine.app_name().is_some() {
                let mut writer = #engine.lsp_cache_writer();
                let data = ::truffle::postcard::to_io(&#engine, &mut writer).unwrap();
                let _ = std::io::Write::flush(&mut writer);
            }
        }
        .into()
    }
    #[cfg(not(feature = "lsp"))]
    {
        quote! {
            if #fun_is_async() {
                #engine.with(#register_fun())
            } else {
                #engine.register_fn(#name, #fun, Some(#fn_location()))
            };
        }
        .into()
    }
}

#[proc_macro_attribute]
pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    foo(item).unwrap().into()
}

fn foo(item: TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
    let input = syn::parse::<ItemFn>(item.clone())?;
    let output = if input.sig.asyncness.is_some() {
        let register_fn = generate::register_fn(input.clone())?;
        let fn_is_async = generate::fn_is_async(input.clone())?;
        let fn_location = generate::fn_location(input.clone())?;

        quote! {
            #input

            #register_fn
            #fn_is_async
            #fn_location
        }
    } else {
        let register_fn = generate::register_fn_stub(input.clone()).expect("stub should generate");
        let fn_is_async = generate::fn_is_async(input.clone())?;
        let fn_location = generate::fn_location(input.clone())?;
        quote! {
            #input

            #register_fn
            #fn_is_async
            #fn_location
        }
    };
    Ok(output)
}

mod generate {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use syn::{
        parse_quote,
        punctuated::Punctuated,
        token::{Comma, Mut},
        FnArg, ItemFn, Pat, ReturnType,
    };

    pub fn register_fn(input: ItemFn) -> Result<TokenStream, syn::Error> {
        let wrapped_fn_name = input.sig.ident.to_string();
        let register_fn_name = format_ident!("register_{wrapped_fn_name}");
        let wrapped_fn = wrapped_fn(input.clone())?;
        let registration_closure = registration_closure(input)?;

        Ok(quote! {
            fn #register_fn_name(name: &'static str) -> impl Fn(&mut truffle::Engine) {
                use futures::FutureExt;

                #wrapped_fn

                #registration_closure
            }
        })
    }

    pub(crate) fn fn_location(input: ItemFn) -> Result<TokenStream, syn::Error> {
        let wrapped_fn_name = input.sig.ident.to_string();
        let mut fn_location = input;
        fn_location.sig.asyncness = None;
        fn_location.sig.ident = format_ident!("{wrapped_fn_name}_location");
        fn_location.sig.output = syn::parse_str("-> &'static std::panic::Location<'static>")
            .expect("this should parse as a return type");
        fn_location.sig.inputs = Default::default();
        fn_location.block = parse_quote! {
            {
                std::panic::Location::caller()
            }
        };
        let tokens = quote! {
            #fn_location
        };
        Ok(tokens)
    }

    pub fn register_fn_stub(input: ItemFn) -> Result<TokenStream, syn::Error> {
        let wrapped_fn_name = input.sig.ident.to_string();
        let mut register_fn_stub = input;
        register_fn_stub.sig.ident = format_ident!("register_{wrapped_fn_name}");
        register_fn_stub.sig.output = syn::parse_str("-> impl Fn(&mut truffle::Engine)")
            .expect("this should parse as a return type");
        let mut inputs: Punctuated<FnArg, Comma> = Default::default();
        let arg = syn::parse_str("name: &'static str").expect("should parse as arguments");
        inputs.push(arg);
        register_fn_stub.sig.inputs = inputs;
        register_fn_stub.block = syn::parse_str(
            "{|engine| unreachable!(\"register fn should only be called for async fns\")}",
        )
        .expect("this should parse as a block body for a function");

        Ok(quote! {
            #register_fn_stub
        })
    }

    pub fn fn_is_async(input: ItemFn) -> Result<TokenStream, syn::Error> {
        let wrapped_fn_name = input.sig.ident.to_string();
        let is_async = input.sig.asyncness.is_some();
        let mut fn_is_async = input;
        fn_is_async.sig.asyncness = None;
        fn_is_async.sig.ident = format_ident!("{wrapped_fn_name}_is_async");
        fn_is_async.sig.output =
            syn::parse_str("-> bool").expect("this should parse as a return type");
        fn_is_async.sig.inputs = Default::default();
        fn_is_async.block = parse_quote! {
            {
                #is_async
            }
        };

        Ok(quote! {
            #fn_is_async
        })
    }

    fn wrapped_fn(input: ItemFn) -> Result<TokenStream, syn::Error> {
        let wrapped_fn_name = input.sig.ident;
        let wrapped_params = input.sig.inputs.iter().map(|arg| {
            let mut arg = arg.clone();
            match &mut arg {
                FnArg::Receiver(_receiver) => todo!(),
                FnArg::Typed(pattype) => {
                    match &mut *pattype.pat {
                        Pat::Ident(patident) => patident.mutability = Some(Mut::default()),
                        _ => todo!(),
                    }

                    pattype.ty = Box::new(
                        syn::parse_str("&'a mut ::truffle::Value")
                            .expect("input should be a valid rust type"),
                    );
                }
            }
            arg
        });

        let idents = input.sig.inputs.iter().map(|arg| match arg {
            FnArg::Receiver(_) => todo!(),
            FnArg::Typed(pattype) => match &*pattype.pat {
                Pat::Ident(patident) => &patident.ident,
                _ => todo!(),
            },
        });

        let converted_args = idents
            .clone()
            .zip(input.sig.inputs.iter())
            .map(|(ident, arg)| {
                let ty = match arg {
                    FnArg::Receiver(_) => todo!(),
                    FnArg::Typed(pattype) => &pattype.ty,
                };
                quote! { let #ident = #ident.downcast_mut::<#ty>().expect("downcast type should match the actual type"); }
            });

        let idents =
            idents
                .into_iter()
                .zip(input.sig.inputs.iter())
                .map(|(ident, arg)| match arg {
                    FnArg::Receiver(_) => todo!(),
                    FnArg::Typed(pattype) => match &*pattype.ty {
                        syn::Type::Reference(_) => quote! { #ident },
                        syn::Type::Path(_) => quote! { #ident.clone() },
                        _ => todo!(),
                    },
                });

        Ok(quote! {
            fn wrapped_fn<'a>(
                #(#wrapped_params),*
            ) -> futures::future::BoxFuture<'a, Result<::truffle::Value, String>> {
                async move {
                    #(
                    #converted_args
                    )*
                    Ok(Box::new(#wrapped_fn_name(#(#idents),*).await) as ::truffle::Value)
                }
                .boxed()
            }
        })
    }

    fn registration_closure(input: ItemFn) -> Result<TokenStream, syn::Error> {
        let wrapped_fn_name = input.sig.ident.to_string();
        let fn_location = format_ident!("{wrapped_fn_name}_location");

        let param_types = input.sig.inputs.iter().map(|arg| {
            match arg {
                FnArg::Receiver(receiver) => &receiver.ty,
                FnArg::Typed(pattype) => &pattype.ty,
            }
        }).map(|ty| {
            quote! { if let Some(id) = engine.get_type::<#ty>() { id } else { engine.register_type::<#ty>() } }
        });

        let ret_type = match input.sig.output {
            ReturnType::Default => syn::parse_str("()")?,
            ReturnType::Type(_, ty) => ty,
        };

        let wrapper = match input.sig.inputs.len() {
            0 => quote! { Function::ExternalAsyncFn0(wrapped_fn) },
            1 => quote! { Function::ExternalAsyncFn1(wrapped_fn) },
            2 => quote! { Function::ExternalAsyncFn2(wrapped_fn) },
            3 => quote! { Function::ExternalAsyncFn3(wrapped_fn) },
            4 => quote! { Function::ExternalAsyncFn4(wrapped_fn) },
            _ => unimplemented!(
                "only async functions with up to 4 arguments are currently supported"
            ),
        };

        Ok(quote! {
            |engine: &mut Engine| {
                use truffle::Function;

                let args = vec![#(#param_types),*];
                engine.add_async_call(
                    args,
                    engine.get_type::<#ret_type>().expect("engine should already know about this type"),
                    #wrapper,
                    name,
                    #fn_location()
                );
            }
        })
    }
}
