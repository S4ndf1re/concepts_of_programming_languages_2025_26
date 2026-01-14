use std::fmt::Display;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Ident, parse_macro_input};

use syn::punctuated::Punctuated;
use syn::{
    Result, Token, braced,
    parse::{Parse, ParseStream},
};

#[derive(Debug)]
struct FunctionAttr {
    name: Ident,
    inputs: Vec<Ident>,
    output: Option<Ident>,
}

impl FunctionAttr {
    fn inputs_to_token_stream(&self) -> proc_macro2::TokenStream {
        let inputs_str = self
            .inputs
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>();

        quote! {
            #( (#inputs_str.to_owned(), parser_types::TypeSymbol::strong(parser_types::TypeSymbolType::Any)) ),*
        }
    }

    fn output_to_token_stream(&self) -> proc_macro2::TokenStream {
        match &self.output {
            Some(_) => {
                quote! {
                    Some(std::boxed::Box::new(parser_types::TypeSymbol::strong(parser_types::TypeSymbolType::Any)))
                }
            }
            None => quote! {None},
        }
    }

    fn to_function_tuple(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let name_str = name.to_string();
        let name_converted = format!("{}_converted", name_str);

        let ident_converted = Ident::new(&name_converted, name.span());

        let inputs = self.inputs_to_token_stream();
        let output = self.output_to_token_stream();

        quote! {
           (
             #name_str.to_owned(),
             parser_types::FunctionType {
                 name: #name_str.to_owned(),
                 is_method: true,
                 params: vec![#inputs],
                 return_type: #output,
                 execution_body: parser_types::FunctionExecutionStrategy::Buildin(Self::#ident_converted),
             },
           )
        }
    }
}

impl Parse for FunctionAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let content;
        let _braces = braced!(content in input);

        let inputs_punctuated: Punctuated<Ident, Token![,]> =
            content.parse_terminated(Ident::parse, Token![,])?;

        input.parse::<Token![->]>()?;

        let output: Option<Ident> = input.parse().ok();

        Ok(FunctionAttr {
            name,
            inputs: inputs_punctuated.into_iter().collect(),
            output,
        })
    }
}

#[proc_macro_derive(BuiltinStruct, attributes(scope, method, function))]
pub fn derive_builtin_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let Data::Struct(ref data) = input.data else {
        panic!("is not a struct");
    };

    let scope_field = data
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("scope")))
        .collect::<Vec<_>>();

    if scope_field.len() != 1 {
        panic!("exactly one scope field must be present");
    }

    let Some(scope_field) = scope_field.first() else {
        panic!("exactly one scope field must be present");
    };
    let scope_field_ident = scope_field.ident.as_ref().unwrap();

    let methods = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("method"))
        .map(|attr| attr.parse_args::<FunctionAttr>().unwrap())
        .collect::<Vec<_>>();

    println!("{methods:?}");

    let statics = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("function"))
        .map(|attr| attr.parse_args::<FunctionAttr>().unwrap())
        .collect::<Vec<_>>();

    let instantiable = proc_macro2::TokenStream::from(build_instantiable(&name, scope_field_ident));
    let scope_like = proc_macro2::TokenStream::from(build_scope_like(
        &name,
        scope_field_ident,
        &methods,
        &statics,
    ));
    let builtin_struct =
        proc_macro2::TokenStream::from(build_builtin_struct(&name, &methods, &statics));

    let output = quote! {
        #scope_like
        #builtin_struct
        #instantiable

    };

    output.into()
}

fn build_instantiable(ident: &Ident, scope_field: &Ident) -> TokenStream {
    let ident_str = ident.to_string();

    quote! {
        impl parser_types::Instantiable for #ident {
            fn instantiate(
                &self,
                local_scope: Rc<RefCell<parser_types::Scope>>,
                params: std::collections::HashMap<parser_types::Symbol, Box<parser_types::InterpreterValue>>,
            ) -> Result<parser_types::InterpreterValue, parser_types::Error> {
                let new_value = Self {
                    #scope_field: local_scope,
                    // TODO: apply more args
                };

                Ok(parser_types::InterpreterValue::Strong(std::rc::Rc::new(std::cell::RefCell::new(
                    parser_types::InterpreterValue::BuiltinStruct(
                        #ident_str.to_owned(),
                        std::rc::Rc::new(std::cell::RefCell::new(new_value)),
                    ),
                ))))
            }

            fn get_required_parameters(&self) -> std::collections::HashMap<parser_types::Symbol, parser_types::TypeSymbol> {
                // is emtpy, as no args are required
                std::collections::HashMap::new()
            }
        }
    }
    .into()
}

fn build_builtin_struct(
    ident: &Ident,
    methods: &[FunctionAttr],
    statics: &[FunctionAttr],
) -> TokenStream {
    let name_str = ident.to_string();

    let methods_tokenstreamed = methods
        .iter()
        .map(|m| m.to_function_tuple())
        .collect::<Vec<_>>();
    let statics_tokenstreamed = statics
        .iter()
        .map(|m| m.to_function_tuple())
        .collect::<Vec<_>>();

    quote! {
        impl parser_types::BuiltinStruct for #ident {

            fn to_type(self) -> Result<parser_types::TypeSymbol, parser_types::Error> {
                Ok(parser_types::TypeSymbol::strong(parser_types::TypeSymbolType::Struct(parser_types::StructType {
                    name: self.name(),
                    methods: vec![
                        #( #methods_tokenstreamed ),*
                    ],
                    statics: vec![
                        #( #statics_tokenstreamed ),*
                    ],
                    fields: vec![],
                    prefab: Some(std::rc::Rc::new(self)),
                })))
            }

            fn name(&self) -> String {
                #name_str.to_owned()
            }

            fn resolve_builtin_type(&self) -> Option<parser_types::TypeSymbol> {
                todo!()
                // self.defining_scope
                //     .borrow()
                //     .resolve_defined_type(&self.name())
            }
        }
    }
    .into()
}

fn build_scope_like(
    ident: &Ident,
    scope: &Ident,
    methods: &[FunctionAttr],
    statics: &[FunctionAttr],
) -> TokenStream {
    let mut allowed_names = Vec::with_capacity(methods.len() + statics.len());

    for method in methods {
        allowed_names.push(method.name.to_string());
    }

    for method in statics {
        allowed_names.push(method.name.to_string());
    }

    quote!{
        impl parser_types::ScopeLike for #ident {
                fn resolve_value(&self, name: &parser_types::Symbol) -> Result<parser_types::InterpreterValue, parser_types::Error> {
                    let is_allowed = match name.as_str() {
                        #( #allowed_names )|* => true,
                        _ => false,
                    };

                    if is_allowed {
                        Ok(parser_types::InterpreterValue::Function(name.clone()))
                    } else {
                        Err(parser_types::Error::SymbolNotFound(name.clone()))
                    }
                }

                fn set_value(&mut self, name: &parser_types::Symbol, _value: parser_types::InterpreterValue) -> Result<(), parser_types::Error> {
                    Err(parser_types::Error::SymbolNotFound(name.clone()))
                }

                fn resolve_type(&self, name: &parser_types::Symbol) -> Result<parser_types::TypeSymbol, parser_types::Error> {
                    let Some(struct_type) = self
                        .#scope
                        .borrow()
                        .resolve_defined_type(&parser_types::BuiltinStruct::name(self))
                    else {
                        return Err(parser_types::Error::SymbolNotFound(parser_types::BuiltinStruct::name(self)));
                    };

                    match &struct_type.type_of {
                        parser_types::TypeSymbolType::Struct(strct) => {
                            let method_result = strct
                                .methods
                                .iter()
                                .find(|f| &f.0 == name)
                                .map(|v| parser_types::TypeSymbol::strong(parser_types::TypeSymbolType::Function(v.1.clone())));

                            if let Some(method) = method_result {
                                return Ok(method);
                            }

                            let static_result = strct
                                .statics
                                .iter()
                                .find(|f| &f.0 == name)
                                .map(|v| parser_types::TypeSymbol::strong(parser_types::TypeSymbolType::Function(v.1.clone())));

                            if let Some(r#static) = static_result {
                                return Ok(r#static);
                            }

                            let field_result = strct
                                .fields
                                .iter()
                                .find(|f| &f.0 == name)
                                .map(|v| v.1.clone());

                            if let Some(field) = field_result {
                                return Ok(field);
                            }

                            Err(parser_types::Error::SymbolNotFound(name.clone()))
                        }
                        _ => Err(parser_types::Error::SymbolNotFound(name.clone())),
                    }
                }

                fn get_outer_scope(&self) -> Result<std::rc::Rc<std::cell::RefCell<parser_types::Scope>>, parser_types::Error> {
                    todo!()
                    // Ok(Rc::clone(&self.defining_scope))
                }
            }
    }.into()
}
