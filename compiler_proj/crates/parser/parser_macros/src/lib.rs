use std::collections::VecDeque;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Field, FnArg, Ident, ImplItem, ItemImpl, Pat, ReturnType, Visibility,
    parse_macro_input,
};

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
            #( (#inputs_str.to_owned(), ::parser_types::TypeSymbol::strong(::parser_types::TypeSymbolType::Any)) ),*
        }
    }

    fn output_to_token_stream(&self) -> proc_macro2::TokenStream {
        match &self.output {
            Some(_) => {
                quote! {
                    Some(std::boxed::Box::new(::parser_types::TypeSymbol::strong(::parser_types::TypeSymbolType::Any)))
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
             ::parser_types::FunctionType {
                 name: #name_str.to_owned(),
                 is_method: true,
                 params: vec![#inputs],
                 return_type: #output,
                 execution_body: ::parser_types::FunctionExecutionStrategy::Buildin(Self::#ident_converted),
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

#[proc_macro_derive(BuiltinStruct, attributes(scope))]
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

    let non_scope_fields = data
        .fields
        .iter()
        .filter(|f| !f.attrs.iter().any(|a| a.path().is_ident("scope")))
        .collect::<Vec<_>>();

    if scope_field.len() != 1 {
        panic!("exactly one scope field must be present");
    }

    let Some(scope_field) = scope_field.first() else {
        panic!("exactly one scope field must be present");
    };
    let scope_field_ident = scope_field.ident.as_ref().unwrap();

    let instantiable = proc_macro2::TokenStream::from(build_instantiable(
        &name,
        scope_field_ident,
        &non_scope_fields,
    ));
    let scope_like = proc_macro2::TokenStream::from(build_scope_like(&name, scope_field_ident));
    let builtin_struct =
        proc_macro2::TokenStream::from(build_builtin_struct(&name, scope_field_ident));

    let output = quote! {
        #scope_like
        #builtin_struct
        #instantiable

    };

    output.into()
}

fn build_instantiable(ident: &Ident, scope_field: &Ident, fields: &[&Field]) -> TokenStream {
    let ident_str = ident.to_string();

    let quoted_fields = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            // let type_of = &field.ty;
            quote! {
                #ident: Default::default(),
            }
        })
        .collect::<Vec<_>>();

    quote! {
        impl ::parser_types::Instantiable for #ident {
            fn instantiate(
                &self,
                local_scope: Rc<RefCell<::parser_types::Scope>>,
                params: std::collections::HashMap<::parser_types::Symbol, Box<::parser_types::InterpreterValue>>,
            ) -> Result<::parser_types::InterpreterValue, ::parser_types::Error> {
                let new_value = Self {
                    #scope_field: local_scope,
                    #( #quoted_fields ),*
                };

                Ok(::parser_types::InterpreterValue::Strong(std::rc::Rc::new(std::cell::RefCell::new(
                    ::parser_types::InterpreterValue::BuiltinStruct(
                        #ident_str.to_owned(),
                        std::rc::Rc::new(std::cell::RefCell::new(new_value)),
                    ),
                ))))
            }

            fn get_required_parameters(&self) -> Result<std::collections::HashMap<::parser_types::Symbol, ::parser_types::TypeSymbol>, ::parser_types::Error> {
                // is emtpy, as no args are required
                Ok(std::collections::HashMap::new())
            }
        }
    }
    .into()
}

fn build_builtin_struct(ident: &Ident, scope: &Ident) -> TokenStream {
    let name_str = ident.to_string();
    quote! {
        impl ::parser_types::BuiltinStruct for #ident {

            fn to_type(self) -> Result<::parser_types::TypeSymbol, ::parser_types::Error> {
                Ok(::parser_types::TypeSymbol::strong(::parser_types::TypeSymbolType::Struct(::parser_types::StructType {
                    name: self.name(),
                    methods: Self::__get_type_methods(),
                    statics: Self::__get_type_statics(),
                    fields: vec![],
                    prefab: Some(std::rc::Rc::new(self)),
                })))
            }

            fn name(&self) -> String {
                #name_str.to_owned()
            }

            fn resolve_builtin_type(&self) -> Option<::parser_types::TypeSymbol> {
                self.#scope
                    .borrow()
                    .resolve_defined_type(&self.name())
            }
        }
    }
    .into()
}

fn build_scope_like(ident: &Ident, scope: &Ident) -> TokenStream {
    quote!{
        impl ::parser_types::ScopeLike for #ident {
                fn resolve_value(&self, name: &::parser_types::Symbol) -> Result<::parser_types::InterpreterValue, ::parser_types::Error> {
                    let is_allowed = Self::__get_allowed_names().contains(name);

                    if is_allowed {
                        Ok(::parser_types::InterpreterValue::Function(name.clone()))
                    } else {
                        Err(::parser_types::Error::SymbolNotFound(name.clone()))
                    }
                }

                fn set_value(&mut self, name: &::parser_types::Symbol, _value: ::parser_types::InterpreterValue) -> Result<(), ::parser_types::Error> {
                    Err(::parser_types::Error::SymbolNotFound(name.clone()))
                }

                fn resolve_type(&self, name: &::parser_types::Symbol) -> Result<::parser_types::TypeSymbol, ::parser_types::Error> {
                    let Some(struct_type) = self
                        .#scope
                        .borrow()
                        .resolve_defined_type(&::parser_types::BuiltinStruct::name(self))
                    else {
                        return Err(::parser_types::Error::SymbolNotFound(::parser_types::BuiltinStruct::name(self)));
                    };

                    match &struct_type.type_of {
                        ::parser_types::TypeSymbolType::Struct(strct) => {
                            let method_result = strct
                                .methods
                                .iter()
                                .find(|f| &f.0 == name)
                                .map(|v| ::parser_types::TypeSymbol::strong(::parser_types::TypeSymbolType::Function(v.1.clone())));

                            if let Some(method) = method_result {
                                return Ok(method);
                            }

                            let static_result = strct
                                .statics
                                .iter()
                                .find(|f| &f.0 == name)
                                .map(|v| ::parser_types::TypeSymbol::strong(::parser_types::TypeSymbolType::Function(v.1.clone())));

                            if let Some(r#static) = static_result {
                                return Ok(r#static);
                            }

                            Err(::parser_types::Error::SymbolNotFound(name.clone()))
                        }
                        _ => Err(::parser_types::Error::SymbolNotFound(name.clone())),
                    }
                }

                fn get_outer_scope(&self) -> Result<std::rc::Rc<std::cell::RefCell<::parser_types::Scope>>, ::parser_types::Error> {
                    Ok(Rc::clone(&self.#scope))
                }
            }
    }.into()
}

fn compare_path(ty: &syn::Type, segments: &[&str]) -> bool {
    let mut buffer = VecDeque::from_iter(segments.iter());
    if let syn::Type::Path(tp) = ty {
        while !buffer.is_empty() {
            if tp.path.segments.len() != buffer.len() {
                buffer.pop_front();
            } else {
                break;
            }
        }

        if buffer.is_empty() {
            return false;
        }

        tp.path
            .segments
            .iter()
            .zip(buffer.iter())
            .all(|(a, b)| a.ident == b)
    } else {
        false
    }
}

#[proc_macro_attribute]
pub fn expose_funcs(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_impl = parse_macro_input!(item as ItemImpl);
    let impl_self_type = &input_impl.self_ty;

    let mut converted_methods = Vec::new();
    let mut function_attrs_methods = Vec::new();
    let mut function_attrs_statics = Vec::new();

    for item in &mut input_impl.items {
        if let ImplItem::Fn(method) = item {
            let fitting_attributes = method
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("expose"))
                .collect::<Vec<_>>();

            let signature = &method.sig;
            if !fitting_attributes.is_empty() && matches!(method.vis, Visibility::Public(_)) {
                let name = &signature.ident;
                let name_str = name.to_string();
                let name_converted = format!("{}_converted", name);
                let ident_converted = Ident::new(&name_converted, name.span());

                let is_method = signature.receiver().is_some();

                let args = &signature.inputs;
                let return_value = &signature.output;

                let mut let_statements = Vec::new();
                let mut arg_names = Vec::new();

                for arg in args {
                    let quoted_arg = match arg {
                        FnArg::Receiver(_) => quote! {
                            let slf = scope.resolve_value(&"self".to_owned())?;
                        },
                        FnArg::Typed(typed_arg) => {
                            if let Pat::Ident(ident) = &*typed_arg.pat {
                                let ident_str = ident.ident.to_string();

                                let mut current_type = &*typed_arg.ty;
                                while let syn::Type::Reference(type_ref) = current_type {
                                    current_type = &*type_ref.elem;
                                }

                                let is_world = compare_path(current_type, &["ecs", "World"]);

                                arg_names.push(&ident.ident);

                                if !is_world {
                                    quote! {
                                        let #ident = scope.resolve_value(&#ident_str.to_owned())?;
                                    }
                                } else {
                                    quote! {
                                        let #ident = world;
                                    }
                                }
                            } else {
                                quote! {}
                            }
                        }
                    };
                    let_statements.push(quoted_arg);
                }

                let function_attr = FunctionAttr {
                    name: name.clone(),
                    inputs: arg_names
                        .iter()
                        .map(|ident| (*ident).clone())
                        .collect::<Vec<_>>(),
                    output: match return_value {
                        ReturnType::Default => None,
                        ReturnType::Type(_, _) => Some(Ident::new("any_type", name.span())),
                    },
                };
                if is_method {
                    function_attrs_methods.push(function_attr);
                } else {
                    function_attrs_statics.push(function_attr);
                }

                let new_method_body = if is_method {
                    quote! {
                        pub fn #ident_converted(scope: std::rc::Rc<std::cell::RefCell<::parser_types::Scope>>, world: &ecs::World) -> Result<::parser_types::IsReturn, ::parser_types::Error> {
                            use ::parser_types::ScopeLike;
                            #( #let_statements )*

                            match &slf.deref_value()? {
                                ::parser_types::InterpreterValue::BuiltinStruct(__name, __ptr) => unsafe {
                                    let __self_val = (&mut *__ptr.borrow_mut() as *mut dyn ::parser_types::BuiltinStruct as *mut Self);
                                    let result = (*__self_val).#name( #( #arg_names ),* )?;
                                    Ok(::parser_types::IsReturn::Return(result))
                                },
                                _ => Err(::parser_types::Error::OperationUnsupported{
                                    operation: #name_str.to_owned(),
                                    type_of: "must be Builtin value".to_owned(),
                                }),
                            }
                        }
                    }
                } else {
                    quote! {
                        pub fn #ident_converted(scope: std::rc::Rc<std::cell::RefCell<::parser_types::Scope>>, world: &ecs::World) -> Result<::parser_types::IsReturn, ::parser_types::Error> {
                            use ::parser_types::ScopeLike;
                            #( #let_statements )*

                            match &slf.deref_value()? {
                                ::parser_types::InterpreterValue::BuiltinStruct(__name, __ptr) => unsafe {
                                    let result = Self::#name( #( #arg_names ),* )?;
                                    Ok(::parser_types::IsReturn::Return(result))
                                },
                                _ => Err(::parser_types::Error::OperationUnsupported{
                                    operation: #name_str.to_owned(),
                                    type_of: "must be Builtin value".to_owned(),
                                }),
                            }
                        }
                    }
                };

                converted_methods.push(new_method_body);
            }

            // remove all expose attributes
            method.attrs = method
                .attrs
                .iter()
                .filter(|attr| !attr.path().is_ident("expose"))
                .cloned()
                .collect::<Vec<_>>();
        }
    }

    // build metadata for function helpers
    let methods_tokenstreamed = function_attrs_methods
        .iter()
        .map(|m| m.to_function_tuple())
        .collect::<Vec<_>>();
    let statics_tokenstreamed = function_attrs_statics
        .iter()
        .map(|m| m.to_function_tuple())
        .collect::<Vec<_>>();

    let mut allowed_names =
        Vec::with_capacity(function_attrs_methods.len() + function_attrs_statics.len());

    for method in &function_attrs_methods {
        allowed_names.push(method.name.to_string());
    }

    for method in &function_attrs_statics {
        allowed_names.push(method.name.to_string());
    }

    quote! {
        #input_impl

        impl #impl_self_type {
            #( #converted_methods )*

            pub fn __get_type_methods() -> Vec<(::parser_types::Symbol, ::parser_types::FunctionType)> {
                vec![#( #methods_tokenstreamed ),*]
            }

            pub fn __get_type_statics() -> Vec<(::parser_types::Symbol, ::parser_types::FunctionType)> {
                vec![#( #statics_tokenstreamed ),*]
            }

            pub fn __get_allowed_names() -> Vec<String> {
                vec![#( #allowed_names.to_owned() ),*]
            }
        }
    }
    .into()
}

#[proc_macro_derive(BuiltinComponent, attributes(scope))]
pub fn derive_builtin_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let name_string = name.to_string();

    let Data::Struct(ref data) = input.data else {
        panic!("is not a struct");
    };

    let scope_field = data
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("scope")))
        .collect::<Vec<_>>();

    let non_scope_fields = data
        .fields
        .iter()
        .filter(|f| !f.attrs.iter().any(|a| a.path().is_ident("scope")))
        .collect::<Vec<_>>();

    if scope_field.len() != 1 {
        panic!("exactly one scope field must be present");
    }

    let Some(scope_field) = scope_field.first() else {
        panic!("exactly one scope field must be present");
    };
    let scope_field_ident = scope_field.ident.as_ref().unwrap();

    let instantiable = proc_macro2::TokenStream::from(build_instantiable_component(
        &name,
        scope_field_ident,
        &non_scope_fields,
    ));
    let scope_like = proc_macro2::TokenStream::from(build_scope_like_component(
        &name,
        scope_field_ident,
        &non_scope_fields,
    ));
    let builtin_struct = proc_macro2::TokenStream::from(build_builtin_component(
        &name,
        scope_field_ident,
        &non_scope_fields,
    ));

    let output = quote! {
        #scope_like
        #builtin_struct
        #instantiable


        impl ::ecs::Component for #name {
            fn get_ident(&self) -> String {
                #name_string.to_owned()
            }
        }
    };

    output.into()
}

fn build_instantiable_component(
    ident: &Ident,
    scope_field: &Ident,
    fields: &[&Field],
) -> TokenStream {
    let ident_str = ident.to_string();

    let quoted_fields = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ident_string = ident.to_string();
            // let type_of = &field.ty;
            quote! {
                #ident: params.get(#ident_string).ok_or(::parser_types::Error::SymbolNotFound(#ident_string.to_owned()))?.as_ref().clone()
            }
        })
        .collect::<Vec<_>>();

    quote! {
        impl ::parser_types::Instantiable for #ident {
            fn instantiate(
                &self,
                local_scope: Rc<RefCell<::parser_types::Scope>>,
                params: std::collections::HashMap<::parser_types::Symbol, Box<::parser_types::InterpreterValue>>,
            ) -> Result<::parser_types::InterpreterValue, ::parser_types::Error> {
                let new_value = Self {
                    #scope_field: local_scope,
                    #( #quoted_fields ),*
                };

                Ok(::parser_types::InterpreterValue::Strong(std::rc::Rc::new(std::cell::RefCell::new(
                    ::parser_types::InterpreterValue::BuiltinComponent(
                        #ident_str.to_owned(),
                        std::rc::Rc::new(std::cell::RefCell::new(new_value)),
                    ),
                ))))
            }

            fn get_required_parameters(&self) -> Result<std::collections::HashMap<::parser_types::Symbol, ::parser_types::TypeSymbol>, ::parser_types::Error> {
                let Some(struct_type) = self
                    .#scope_field
                    .borrow()
                    .resolve_defined_type(&::parser_types::BuiltinComponent::name(self))
                else {
                    return Err(::parser_types::Error::SymbolNotFound(::parser_types::BuiltinComponent::name(self)));
                };

                match &struct_type.type_of {
                    ::parser_types::TypeSymbolType::Component(comp) => {
                        let field_result = comp
                            .fields
                            .iter()
                            .map(|v| (v.0.clone(), v.1.clone()))
                            .collect::<std::collections::HashMap<_, _>>();

                        Ok(field_result)
                    }
                    _ => Err(::parser_types::Error::SymbolNotFound(::parser_types::BuiltinComponent::name(self))),
                }
            }
        }
    }
    .into()
}

fn build_builtin_component(ident: &Ident, scope: &Ident, fields: &[&Field]) -> TokenStream {
    let name_str = ident.to_string();
    let quoted_fields = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ident_string = ident.to_string();
            quote! {
                (#ident_string.to_owned(), ::parser_types::TypeSymbol::strong(::parser_types::TypeSymbolType::Any))
            }
        })
        .collect::<Vec<_>>();
    quote! {
        impl ::parser_types::BuiltinComponent for #ident {

            fn to_type(self) -> Result<::parser_types::TypeSymbol, ::parser_types::Error> {
                Ok(::parser_types::TypeSymbol::strong(::parser_types::TypeSymbolType::Component(::parser_types::ComponentType {
                    name: self.name(),
                    fields: vec![#( #quoted_fields ),*],
                    prefab: Some(std::rc::Rc::new(self)),
                })))
            }

            fn name(&self) -> String {
                #name_str.to_owned()
            }

            fn resolve_builtin_type(&self) -> Option<::parser_types::TypeSymbol> {
                self.#scope
                    .borrow()
                    .resolve_defined_type(&self.name())
            }
        }
    }
    .into()
}

fn build_scope_like_component(ident: &Ident, scope: &Ident, fields: &[&Field]) -> TokenStream {
    let quoted_assigns = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ident_string = ident.to_string();

            quote! {
                if name == #ident_string {
                    self.#ident = value;
                    return Ok(())
                }
            }
        })
        .collect::<Vec<_>>();

    let quoted_get = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ident_string = ident.to_string();
            quote! {
                if name == #ident_string {
                    return Ok(self.#ident.clone())
                }
            }
        })
        .collect::<Vec<_>>();

    quote!{
        impl ::parser_types::ScopeLike for #ident {
                fn resolve_value(&self, name: &::parser_types::Symbol) -> Result<::parser_types::InterpreterValue, ::parser_types::Error> {
                    #( #quoted_get )*
                    Err(::parser_types::Error::SymbolNotFound(name.clone()))
                }

                fn set_value(&mut self, name: &::parser_types::Symbol, value: ::parser_types::InterpreterValue) -> Result<(), ::parser_types::Error> {
                    #( #quoted_assigns )*
                    Err(::parser_types::Error::SymbolNotFound(name.clone()))
                }

                fn resolve_type(&self, name: &::parser_types::Symbol) -> Result<::parser_types::TypeSymbol, ::parser_types::Error> {
                    let Some(struct_type) = self
                        .#scope
                        .borrow()
                        .resolve_defined_type(&::parser_types::BuiltinComponent::name(self))
                    else {
                        return Err(::parser_types::Error::SymbolNotFound(::parser_types::BuiltinComponent::name(self)));
                    };

                    match &struct_type.type_of {
                        ::parser_types::TypeSymbolType::Component(comp) => {
                            let field_result = comp
                                .fields
                                .iter()
                                .find(|f| &f.0 == name)
                                .map(|v| v.1.clone());

                            if let Some(field) = field_result {
                                return Ok(field);
                            }

                            Err(::parser_types::Error::SymbolNotFound(name.clone()))
                        }
                        _ => Err(::parser_types::Error::SymbolNotFound(name.clone())),
                    }
                }

                fn get_outer_scope(&self) -> Result<std::rc::Rc<std::cell::RefCell<::parser_types::Scope>>, ::parser_types::Error> {
                    Ok(Rc::clone(&self.#scope))
                }
            }
    }.into()
}
