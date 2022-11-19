use std::convert::identity;

use proc_macro2::{TokenStream,};
use syn::{*, punctuated::Punctuated, spanned::Spanned};
use quote::{quote, format_ident, ToTokens, quote_spanned};

mod def;
use def::*;

mod imp;
use imp::*;

#[proc_macro_attribute]
pub fn async_trait_def (_attrs: proc_macro::TokenStream, items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let AsyncTraitDef { attrs, vis, unsafety, auto_token, trait_token, ident, generics, colon_token, supertraits, items, .. } = parse_macro_input!(items as AsyncTraitDef);
    let (impl_generics, _, where_generics) = generics.split_for_impl();

    let tys = items.iter().filter_map(|x| match x {
        AsyncTraitItem::Type(x) => Some(x),
        _ => None
    }).cloned().collect::<Vec<_>>();

    let (items, extras) = items.into_iter().map(|x| define_fn(&vis, &ident, &generics, &tys, x)).unzip::<_, _, Vec<_>, Vec<_>>();
    let extras = extras.into_iter().flat_map(identity).collect::<TokenStream>();

    quote! {
        #(#attrs)*
        #vis #unsafety #auto_token #trait_token #ident #impl_generics #colon_token #supertraits #where_generics {            
            #(#items)*
        }

        #extras
    }.into()
}

#[proc_macro_attribute]
pub fn async_trait_impl (_attrs: proc_macro::TokenStream, items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let AsyncItemImpl { attrs, defaultness, unsafety, impl_token, generics, trait_, self_ty, items, .. } = parse_macro_input!(items as AsyncItemImpl);
    let items = items.into_iter().map(impl_fn);
    let trait_ = match trait_ {
        Some((x, y, z)) => Some(quote!(#x #y #z)),
        None => None
    };

    quote! {
        #(#attrs)*
        #defaultness #unsafety #impl_token #generics #trait_ #self_ty {
            #(#items)*
        }
    }.into()
}

#[inline]
fn define_fn (vis: &Visibility, trait_ident: &Ident, trait_generics: &Generics, trait_tys: &[TraitItemType], sig: AsyncTraitItem) -> (TokenStream, Option<TokenStream>) {
    return match sig {
        AsyncTraitItem::Method(method) if method.sig.asyncness.is_some() => define_async_fn(vis, trait_ident, trait_generics, trait_tys, method),
        other => (other.to_token_stream(), None)
    }
}

fn define_async_fn (vis: &Visibility, trait_ident: &Ident, trait_generics: &Generics, trait_tys: &[TraitItemType], AsyncTraitItemMethod { attrs, sig: Signature { constness, unsafety, abi, fn_token, ident, mut generics, mut inputs, variadic, output, .. }, default, semi_token }: AsyncTraitItemMethod) -> (TokenStream, Option<TokenStream>) {
    let future_name = format_ident!("{}", to_pascal_case(&ident.to_string()));
    let future_output = match output {
        ReturnType::Default => Box::new(parse_quote! { () }),
        ReturnType::Type(_, ty) => ty
    };

    let life = future_generics(inputs.iter_mut(), &mut generics);
    let add_token = match life.is_empty() {
        true => None,
        false => Some(<Token![+]>::default())
    };

    let (impl_generics, ty_generics, where_generics) = generics.split_for_impl();
    let (future_default, default_block, default_impl_struct) = match default {
        Some(x) => {
            let (default_ident, mut default_generics, default_args, default_struct_impl) = define_async_default(
                &attrs, &vis, &ident, &future_name,
                trait_ident, trait_generics, trait_tys, &generics,
                &inputs, &future_output, x
            );

            let first = default_generics.type_params_mut().next().unwrap();
            first.eq_token = Some(Default::default());
            first.default = Some(parse_quote! { Self });

            (
                Some(quote! { = <#default_ident #default_generics as ::core::ops::FnOnce<#default_args>>::Output }),
                Some(quote! {{ todo!() }}),
                Some(default_struct_impl)
            )
        },
        None => (None, None, None)
    };
    
    let tokens = quote! {
        type #future_name #impl_generics: #life #add_token ::core::future::Future<Output = #future_output> #future_default #where_generics;

        #(#attrs)*
        #constness #unsafety #abi #fn_token #ident #impl_generics (#inputs #variadic) -> Self::#future_name #ty_generics #where_generics #default_block #semi_token
    };

    return (tokens, default_impl_struct)
}

fn define_async_default (attrs: &[Attribute], vis: &Visibility, fn_ident: &Ident, fn_pascal_ident: &Ident, trait_ident: &Ident, trait_generics: &Generics, trait_tys: &[TraitItemType], generics: &Generics, args: &Punctuated<FnArg, Token![,]>, output: &Type, block: Block) -> (Ident, Generics, TokenStream, TokenStream) {
    let struct_name = format_ident!("{trait_ident}{fn_pascal_ident}Default");

    // Sealed
    let sealed_ident = format_ident!("{fn_ident}_sealed");
    let extra_ident = format_ident!("{struct_name}Ext");

    // Generics
    let mut struct_generics = trait_generics.clone();
    struct_generics.params.extend(generics.params.iter().cloned());
    
    let generic_name = format_ident!("This");
    struct_generics.params.insert(0, parse_quote! { #generic_name: #trait_ident });

    for TypeParam { bounds, .. } in struct_generics.type_params_mut() {
        if !bounds.iter().any(|x| x == &parse_quote! { Sized }) {
            bounds.push(parse_quote! { ?Sized })
        }
    }

    let (impl_generics, ty_generics, where_generics) = struct_generics.split_for_impl();

    // Arguments
    let mut unnamed_args = Vec::with_capacity(args.len());
    let mut arg_names = Vec::with_capacity(args.len());

    for arg in args.iter() {
        match arg {
            FnArg::Receiver(Receiver { attrs, reference, mutability, .. }) => {
                let (and_token, reference) = match reference {
                    Some((x, y)) => (Some(x), Some(y)),
                    None => (None, None)
                };

                unnamed_args.push(quote! { #(#attrs)* #and_token #reference #mutability #generic_name });
                arg_names.push(quote! { #(#attrs)* this });
            },

            FnArg::Typed(PatType { attrs, pat, ty, .. }) => {
                unnamed_args.push(quote! { #(#attrs)* #ty });
                arg_names.push(quote! { #(#attrs)* #pat });
            }
        }
    }

    // Fields
    let mut struct_fields = Punctuated::<_, Token![,]>::new();
    let mut fields_new = Punctuated::<_, Token![,]>::new();

    for lt @ LifetimeDef { attrs, lifetime, .. } in struct_generics.lifetimes() {
        struct_fields.push(
            quote_spanned! { lt.span() => #(#attrs)* ::core::marker::PhantomData<&#lifetime ()> }
        );

        fields_new.push(
            quote_spanned! { lt.span() => #(#attrs)* ::core::marker::PhantomData }
        );
    }

    for ty @ TypeParam { attrs, ident, .. } in struct_generics.type_params() {        
        struct_fields.push(
            quote_spanned! { ty.span() => #(#attrs)* ::core::marker::PhantomData<#ident> }
        );

        fields_new.push(
            quote_spanned! { ty.span() => #(#attrs)* ::core::marker::PhantomData }
        );
    }

    // Associated types
    let mut struct_tys_defs = Vec::with_capacity(trait_tys.len());
    let mut struct_tys_impls = Vec::with_capacity(trait_tys.len());

    for ty @ TraitItemType { attrs, type_token, ident, generics, semi_token, .. } in trait_tys {
        let (impl_generics, _, _) = generics.split_for_impl();
        struct_tys_defs.push(ty);
        struct_tys_impls.push(quote! {  #(#attrs)* #type_token #ident #impl_generics = <#generic_name as #trait_ident>::#ident #semi_token });
    }

    let tokens = quote! {
        #vis struct #struct_name #impl_generics (#struct_fields);

        #[doc(hidden)]
        mod #sealed_ident {
            pub trait Sealed {}
        }

        #vis trait #extra_ident #impl_generics: #sealed_ident::Sealed #where_generics {
            #(#struct_tys_defs)*
            type __FnOnceOutput__: ::core::future::Future<Output = #output>;

            extern "rust-call" fn __call__(self, args: (#(#unnamed_args,)*)) -> Self::__FnOnceOutput__;
        }

        impl #impl_generics #extra_ident #ty_generics for #struct_name #ty_generics #where_generics {
            #(#struct_tys_impls)*
            type __FnOnceOutput__ = impl ::core::future::Future<Output = #output>;

            #(#attrs)*
            extern "rust-call" fn __call__ (self, (#(#arg_names,)*): (#(#unnamed_args,)*)) -> Self::__FnOnceOutput__ {
                return async move #block
            }
        }

        impl #impl_generics #sealed_ident::Sealed for #struct_name #ty_generics #where_generics {}

        #[doc(hidden)]
        impl #impl_generics #struct_name #ty_generics #where_generics {
            #[inline]
            fn new () -> Self {
                return Self (#fields_new)
            }
        }

        impl #impl_generics ::core::ops::FnOnce<(#(#unnamed_args,)*)> for #struct_name #ty_generics #where_generics {
            type Output = <Self as #extra_ident #ty_generics>::__FnOnceOutput__;

            #[inline(always)]
            extern "rust-call" fn call_once (self, args: (#(#unnamed_args,)*)) -> Self::Output {
                return self.__call__(args)
            }
        }
    };

    return (struct_name, struct_generics, quote! { (#(#unnamed_args,)*) }, tokens)
}

#[inline]
fn impl_fn (sig: ImplItem) -> TokenStream {
    return match sig {
        ImplItem::Method(method) if method.sig.asyncness.is_some() => impl_async_fn(method),
        other => other.to_token_stream()
    }
}

fn impl_async_fn (ImplItemMethod { attrs, vis, defaultness, sig: Signature { constness, asyncness, unsafety, abi, fn_token, ident, mut generics, mut inputs, variadic, output, .. }, block }: ImplItemMethod) -> TokenStream {
    let future_name = format_ident!("{}", to_pascal_case(&ident.to_string()));
    let future_output = match output {
        ReturnType::Default => Box::new(parse_quote! { () }),
        ReturnType::Type(_, ty) => ty
    };

    let life = future_generics(inputs.iter_mut(), &mut generics);
    let add_token = match life.is_empty() {
        true => None,
        false => Some(<Token![+]>::default())
    };
    let (impl_generics, ty_generics, where_generics) = generics.split_for_impl();

    quote! {
        type #future_name #impl_generics = impl #life #add_token ::core::future::Future<Output = #future_output> #where_generics;

        #(#attrs)*
        #vis #defaultness #constness #unsafety #abi #fn_token #ident #impl_generics (#inputs #variadic) -> Self::#future_name #ty_generics #where_generics {
            return #asyncness move #block
        }
    }
}

fn future_generics<'a> (inputs: impl IntoIterator<Item = &'a mut FnArg>, fn_generics: &mut Generics) -> Punctuated<TokenStream, Token![+]> {
    // Reciever generics
    for input in inputs {
        if let FnArg::Receiver(Receiver { attrs, reference, .. }) = input {            
            match reference {
                Some((x, lt @ None)) => {
                    let lifetime: Lifetime = parse_quote_spanned! { x.span => '__self__ };
                    *lt = Some(lifetime.clone());

                    fn_generics.params.insert(0, LifetimeDef {
                        attrs: attrs.clone(),
                        lifetime: lifetime.clone(),
                        colon_token: Default::default(),
                        bounds: Default::default(),
                    }.into());

                    fn_generics.make_where_clause().predicates.push(parse_quote! { Self: #lt });
                },

                Some((_, Some(lt))) => {
                    fn_generics.make_where_clause().predicates.push(parse_quote! { Self: #lt });
                },

                _ => {}
            }
            
            break;
        }
    }

    return fn_generics.lifetimes().map(|LifetimeDef { attrs, lifetime, .. }| 
        quote! { #(#attrs)* #lifetime }
    ).collect();
}

fn to_pascal_case (s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut upper = true;

    for c in s.chars() {
        match c {
            '_' => upper = true,
            c if upper => {
                result.extend(c.to_uppercase());
                upper = false
            },
            c => result.push(c)
        }
    }

    return result
}