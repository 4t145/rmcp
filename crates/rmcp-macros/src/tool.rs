use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    parse::Parse, parse2, parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Expr, FnArg, Ident, Item, ItemFn, LitStr, MetaList, Pat, PatIdent, PatType, Token, Type
};
#[derive(Default)]
struct ToolFnItemAttrs {
    name: Option<Expr>,
    description: Option<Expr>,
}

impl Parse for ToolFnItemAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut description = None;
        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "name" => {
                    let value: Expr = input.parse()?;
                    name = Some(value);
                }
                "description" => {
                    let value: Expr = input.parse()?;
                    description = Some(value);
                }
                _ => {
                    return Err(syn::Error::new(key.span(), "unknown attribute"));
                }
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(ToolFnItemAttrs { name, description })
    }
}

struct ToolFnParamAttrs {
    serde_meta: Vec<MetaList>,
    schemars_meta: Vec<MetaList>,
    ident: Ident,
    rust_type: Box<Type>,
}

impl ToTokens for ToolFnParamAttrs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let rust_type = &self.rust_type;
        let serde_meta = &self.serde_meta;
        let schemars_meta = &self.schemars_meta;
        tokens.extend(quote! {
            #(#[#serde_meta])*
            #(#[#schemars_meta])*
            pub #ident: #rust_type,
        });
    }
}

#[derive(Default)]

enum ToolParams {
    Aggregated {
        rust_type: PatType,
    },
    Params {
        attrs: Vec<ToolFnParamAttrs>,
        trailing_args: Vec<FnArg>
    },
    #[default]
    NoParam,
}

#[derive(Default)]
struct ToolAttrs {
    fn_item: ToolFnItemAttrs,
    params: ToolParams,
}
const TOOL_IDENT: &str = "tool";
const SERDE_IDENT: &str = "serde";
const SCHEMARS_IDENT: &str = "schemars";
const PARAM_IDENT: &str = "param";
const AGGREGATED_IDENT: &str = "aggr";
const REQ_IDENT: &str = "req";

pub enum ParamMarker {
    Param,
    Aggregated,
}

impl Parse for ParamMarker {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            PARAM_IDENT => Ok(ParamMarker::Param),
            AGGREGATED_IDENT | REQ_IDENT => Ok(ParamMarker::Aggregated),
            _ => Err(syn::Error::new(ident.span(), "unknown attribute")),
        }
    }
}

pub(crate) fn tool(attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let mut attrs = ToolAttrs::default();
    let args: ToolFnItemAttrs = syn::parse2(attr)?;
    attrs.fn_item = args;
    let mut input_fn = syn::parse2::<ItemFn>(input)?;
    let mut fommated_fn_args: Punctuated<FnArg, Comma> = Punctuated::new();
    for mut fn_arg in input_fn.sig.inputs {
        enum Catched {
            Param(ToolFnParamAttrs),
            Aggregated(PatType),
        }
        let mut catched = None;
        match &mut fn_arg {
            FnArg::Receiver(_) => {
                fommated_fn_args.push(fn_arg);
                continue;
            }
            FnArg::Typed(pat_type) => {
                let mut serde_metas = Vec::new();
                let mut schemars_metas = Vec::new();
                let mut arg_ident = match pat_type.pat.as_ref() {
                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                    _ => None,
                };
                let raw_attrs: Vec<_> = pat_type.attrs.drain(..).collect();
                for attr in raw_attrs {
                    match &attr.meta {
                        syn::Meta::List(meta_list) => {
                            if meta_list.path.is_ident(TOOL_IDENT) {
                                let pat_type = pat_type.clone();
                                let marker = meta_list.parse_args::<ParamMarker>()?;
                                match marker {
                                    ParamMarker::Param => {
                                        let Some(arg_ident) = arg_ident.take() else {
                                            return Err(syn::Error::new(
                                                proc_macro2::Span::call_site(),
                                                "input param must have an ident as name",
                                            ));
                                        };
                                        catched.replace(Catched::Param(ToolFnParamAttrs {
                                            serde_meta: Vec::new(),
                                            schemars_meta: Vec::new(),
                                            ident: arg_ident,
                                            rust_type: pat_type.ty.clone(),
                                        }));
                                    }
                                    ParamMarker::Aggregated => {
                                        catched.replace(Catched::Aggregated(pat_type.clone()));
                                    }
                                }
                            } else if meta_list.path.is_ident(SERDE_IDENT) {
                                serde_metas.push(meta_list.clone());
                            } else if meta_list.path.is_ident(SCHEMARS_IDENT) {
                                schemars_metas.push(meta_list.clone());
                            } else {
                                pat_type.attrs.push(attr);
                            }
                        }
                        _ => {
                            pat_type.attrs.push(attr);
                        }
                    }
                }
                match catched {
                    Some(Catched::Param(mut param)) => {
                        param.serde_meta = serde_metas;
                        param.schemars_meta = schemars_metas;
                        match &mut attrs.params {
                            ToolParams::Params { attrs, trailing_args } => {
                                attrs.push(param);
                                trailing_args.push(fn_arg);
                            }
                            _ => {
                                attrs.params = ToolParams::Params { attrs: vec![param], trailing_args: vec![fn_arg] };
                            }
                        }
                    }
                    Some(Catched::Aggregated(rust_type)) => {
                        attrs.params = ToolParams::Aggregated { rust_type };
                    }
                    None => {
                        fommated_fn_args.push(fn_arg);
                    }
                }
            }
        }
    }

    input_fn.sig.inputs = fommated_fn_args;
    let name = if let Some(expr) = attrs.fn_item.name {
        expr
    } else {
        let fn_name = &input_fn.sig.ident;
        parse_quote! {
            stringify!(#fn_name)
        }
    };
    let tool_attr_fn_ident = Ident::new(
        &format!("{}_tool_attr", input_fn.sig.ident),
        proc_macro2::Span::call_site(),
    );

    // generate get tool attr function
    let tool_attr_fn = {
        let description = if let Some(expr) = attrs.fn_item.description {
            expr
        } else {
            parse_quote! {
                ""
            }
        };
        let schema = match &attrs.params {
            ToolParams::Aggregated { rust_type } => {
                let schema = quote! {
                    rmcp::handler::server::tool::cached_schema_for_type::<#rust_type>()
                };
                schema
            }
            ToolParams::Params { attrs, .. } => {
                let (param_type, temp_param_type_name) =
                    create_request_type(attrs, input_fn.sig.ident.to_string());
                let schema = quote! {
                    {
                        #param_type
                        rmcp::handler::server::tool::cached_schema_for_type::<#temp_param_type_name>()
                    }
                };
                schema
            }
            ToolParams::NoParam => {
                quote! {
                    rmcp::model::JsonObject::new()
                }
            }
        };
        let input_fn_attrs = &input_fn.attrs;
        let input_fn_vis = &input_fn.vis;
        quote! {
            #(#input_fn_attrs)*
            #input_fn_vis fn #tool_attr_fn_ident() -> rmcp::model::Tool {
                rmcp::model::Tool {
                    name: #name.into(),
                    description: #description.into(),
                    input_schema: #schema.into(),
                }
            }
        }
    };

    // generate wrapped tool function
    let wrapped_tool_function = {
        match &mut attrs.params {
            ToolParams::Aggregated { rust_type } => {
                rust_type.ty = parse_quote! {
                    rmcp::handler::tool::Parameters(#rust_type.ty)
                };
                let PatType {
                    attrs,
                    pat,
                    colon_token,
                    ty,
                } = rust_type;
                input_fn.sig.inputs.push(parse_quote! {
                    #(#attrs)*
                    rmcp::handler::tool::Parameters(#pat): rmcp::handler::tool::Parameters(#ty)
                });
                
                input_fn.into_token_stream()
            }
            ToolParams::Params { attrs, trailing_args } => {
                let wapper_fn_vis = input_fn.vis.clone();
                let mut wapper_fn_sig = input_fn.sig.clone();
                wapper_fn_sig.inputs.push(parse_quote! {
                    __rmcp_tool_req: rmcp::model::JsonObject
                });
                wapper_fn_sig.output = parse_quote! {
                    -> std::result::Result<rmcp::model::CallToolResult, rmcp::Error>
                };
                // rename the wrapper ident
                let wapper_fn_ident = Ident::new(
                    &format!("{}_call", wapper_fn_sig.ident),
                    proc_macro2::Span::call_site(),
                );

                wapper_fn_sig.ident = wapper_fn_ident;


                let (param_type, temp_param_type_name) =
                    create_request_type(attrs, input_fn.sig.ident.to_string());
                
                let params_ident = attrs.iter().map(|attr| &attr.ident).collect::<Vec<_>>();
                // restore raw funcion args
                input_fn.sig.inputs.extend(trailing_args.drain(..));
                
                // has receiver type?



                let ItemFn {
                    attrs: input_fn_attrs,
                    vis: _,
                    sig,
                    block: _,
                } = &input_fn;
                let input_fn_ident = &sig.ident;
                let input_fn_args_ident = sig
                    .inputs
                    .iter()
                    .map(|attr| match attr {
                        FnArg::Typed(pat_type) => pat_type.pat.to_token_stream(),
                        FnArg::Receiver(receiver) => receiver.self_token.into_token_stream(),
                    })
                    .collect::<Vec<_>>();
                if sig.asyncness.is_some() {
                    quote! {
                        #input_fn
                        #(#input_fn_attrs)*
                        #wapper_fn_vis #wapper_fn_sig {
                            use rmcp::handler::server::tool::*;
                            #param_type
                            let #temp_param_type_name {
                                #(#params_ident,)*
                            } = parse_json_object(__rmcp_tool_req)?;
                            Self::#input_fn_ident(
                                #(#input_fn_args_ident,)*
                            ).await.into_call_tool_result()
                        }
                    }
                } else {
                    quote! {
                        #input_fn
                        #(#input_fn_attrs)*
                        #wapper_fn_vis #wapper_fn_sig {
                            use rmcp::handler::server::tool::*;
                            #param_type
                            let #temp_param_type_name {
                                #(#params_ident,)*
                            } = parse_json_object(__rmcp_tool_req)?;
                            Self::#input_fn_ident(
                                #(#input_fn_args_ident,)*
                            ).into_call_tool_result()
                        }
                    }
                }
            }
            ToolParams::NoParam => input_fn.into_token_stream(),
        }
    };
    Ok(quote! {
        #tool_attr_fn
        #wrapped_tool_function
    })
}

fn create_request_type(attrs: &[ToolFnParamAttrs], tool_name: String) -> (TokenStream, Ident) {
    let pascal_case_tool_name = tool_name.to_ascii_uppercase();
    let temp_param_type_name = Ident::new(
        &format!("__{pascal_case_tool_name}ToolCallParam",),
        proc_macro2::Span::call_site(),
    );
    (
        quote! {
            use rmcp::{serde, schemars};
            #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
            pub struct #temp_param_type_name {
                #(#attrs)*
            }
        },
        temp_param_type_name,
    )
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_tool_sync_macro() -> syn::Result<()> {
        let attr = quote! {
            name = "test_tool",
            description = "test tool"
        };
        let input = quote! {
            fn echo(
                &self,  
                #[tool(param)]
                echo: String
            ) -> Result<CallToolResult, McpError> {
                Ok(CallToolResult::success(vec![Content::text(
                    echo
                )]))
            }
        };
        let input = tool(attr, input)?;

        println!("input: {:#}", input);
        Ok(())
    }
}
