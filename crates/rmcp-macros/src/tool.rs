use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Expr, Ident, ItemFn, LitStr, Token, parse::Parse, parse_macro_input, parse2};

struct ToolArgs {
    ident: Ident,
    name: Option<Expr>,
    description: Option<Expr>,
}

impl Parse for ToolArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut description = None;
        let ident: Ident = input.parse()?;
        // while !input.is_empty() {
        //     input.parse::<Token![,]>()?;
        //     let key: Ident = input.parse()?;
        //     input.parse::<Token![=]>()?;

        //     match key.to_string().as_str() {
        //         "name" => {
        //             let value: Expr = input.parse()?;
        //             name = Some(value);
        //         }
        //         "description" => {
        //             let value: Expr = input.parse()?;
        //             description = Some(value);
        //         }
        //         _ => {
        //             return Err(syn::Error::new(key.span(), "unknown attribute"));
        //         }
        //     }
        // }

        Ok(ToolArgs {
            ident,
            name,
            description,
        })
    }
}

pub(crate) fn tool(attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let args: ToolArgs = syn::parse2(attr)?;
    let input_fn = syn::parse2::<ItemFn>(input)?;
    let description = if let Some(args) = args.description {
        args
    } else {
        let doc_strings: Vec<String> = input_fn
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .filter_map(|attr| {
                attr.parse_args::<LitStr>()
                    .ok()
                    .map(|lit| lit.value().trim().to_string())
            })
            .collect();
        let description = doc_strings.join("\n");
        syn::Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit: syn::Lit::Str(LitStr::new(&description, proc_macro2::Span::call_site())),
        })
    };
    let name = if let Some(args) = args.name {
        args
    } else {
        syn::Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit: syn::Lit::Str(LitStr::new(
                &input_fn.sig.ident.to_string(),
                proc_macro2::Span::call_site(),
            )),
        })
    };
    let ident = args.ident;
    let is_async = input_fn.sig.asyncness.is_some();
    let vis = input_fn.vis.clone();
    let input_args = input_fn.sig.inputs.clone();
    if input_args.len() != 1 {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected exactly one argument",
        ));
    }
    let input_arg_type = input_args.first().unwrap();
    let input_arg_type = match input_arg_type {
        syn::FnArg::Typed(pat) => pat.ty.clone(),
        _ => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "expected exactly one argument",
            ));
        }
    };

    let function = input_fn.sig.ident.clone();
    let output = input_fn.sig.output.clone();
    let return_type = match output {
        syn::ReturnType::Type(_, ty) => ty,
        _ => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "expected return type",
            ));
        }
    };
    println!("output: {:?}, {:?}, {}", return_type, input_arg_type, function);
    let output_tokens = if is_async {
        quote! {
            #input_fn
            #vis static #ident:
            _
             = rmcp::handler::server::tool::FunctionalTool::<
                rmcp::handler::server::tool::_AdapterAsyncSingleParam<
                    #input_arg_type,
                    #return_type,
                    _,
                >, _
            >::new_async_single_param(
                std::borrow::Cow::Borrowed(#name),
                std::borrow::Cow::Borrowed(#description),
                #function,
            );
        }
    } else {
        quote! {
            #input_fn
            #vis static #ident = rmcp::handler::server::tool::FunctionalTool::new(
                std::borrow::Cow::Borrowed(#name),
                std::borrow::Cow::Borrowed(#description),
                #function,
            );
        }
    };

    Ok(output_tokens)
}
