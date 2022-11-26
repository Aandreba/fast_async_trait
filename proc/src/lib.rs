use proc_macro2::{TokenStream,};
use syn::{*, punctuated::Punctuated, spanned::Spanned};
use quote::{quote, format_ident, ToTokens};

mod def;
use def::*;

mod imp;
use imp::*;

#[proc_macro_attribute]
pub fn async_trait_def (_attrs: proc_macro::TokenStream, items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let AsyncTraitDef { attrs, vis, unsafety, auto_token, trait_token, ident, generics, colon_token, supertraits, items, .. } = parse_macro_input!(items as AsyncTraitDef);
    let (impl_generics, _, where_generics) = generics.split_for_impl();

    let (items, extra) = items.into_iter()
        .map(|x| define_fn(&vis, &ident, x))
        .unzip::<_, _, Vec<_>, Vec<_>>();

    let extra = extra.into_iter()
        .filter_map(core::convert::identity)
        .collect::<TokenStream>();

    quote! {
        #(#attrs)*
        #vis #unsafety #auto_token #trait_token #ident #impl_generics #colon_token #supertraits #where_generics {            
            #(#items)*
        }

        #extra
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
fn define_fn (vis: &Visibility, trait_ident: &Ident, sig: AsyncTraitItem) -> (TokenStream, Option<TokenStream>) {
    return match sig {
        AsyncTraitItem::Method(method) if method.sig.asyncness.is_some() => define_async_fn(vis, trait_ident, method),
        other => (other.to_token_stream(), None)
    }
}

fn define_async_fn (vis: &Visibility, trait_ident: &Ident, AsyncTraitItemMethod { attrs, sig: Signature { constness, asyncness, unsafety, abi, fn_token, ident, mut generics, mut inputs, variadic, output, .. }, default, semi_token }: AsyncTraitItemMethod) -> (TokenStream, Option<TokenStream>) {
    let future_name = format_ident!("{}", to_pascal_case(&ident.to_string()));
    let future_output = match output {
        ReturnType::Default => Box::new(parse_quote! { () }),
        ReturnType::Type(_, ty) => ty
    };
    
    let (life, main_lt) = future_generics(inputs.iter_mut(), &mut generics);
    if generics.lifetimes().count() > 1 {
        return (syn::Error::new(generics.lifetimes().nth(1).unwrap().span(), "Currently only one lifetime per future is supported").into_compile_error(), None);
    };
    
    let add_token = match life.is_empty() {
        true => None,
        false => Some(<Token![+]>::default())
    };
    let (impl_generics, ty_generics, where_generics) = generics.split_for_impl();

    let (future_default, return_type, extra) = match default {
        Some(block) => {
            let ty_ident = format_ident!("{trait_ident}{future_name}Default");

            let ty_sized = match &main_lt {
                Some(_) => Some(quote! { ?::core::marker::Sized + }),
                None => None
            };

            let ty_lt = match generics.lifetimes().next() {
                Some(LifetimeDef { lifetime, .. }) => Some(quote! { #lifetime + }),
                None => None
            };

            let mut ty_generics = generics.clone();
            ty_generics.params.insert(0, parse_quote! { This: #ty_sized #ty_lt #trait_ident });
            let (impl_ty_generics, _, _) = ty_generics.split_for_impl();

            let tokens = quote! {{
                return #asyncness move #block
            }};

            let opaque = quote! {
                #[doc(hidden)]
                #vis type #ty_ident #impl_ty_generics = impl #ty_lt ::core::future::Future; 
            };

            /*if generics.lifetimes().count() > 0 {
                panic!("{opaque}");
            }*/

            *ty_generics.params.first_mut().unwrap() = GenericParam::Type(TypeParam {
                attrs: Default::default(),
                ident: format_ident!("Self"),
                colon_token: Default::default(),
                bounds: Default::default(),
                eq_token: Default::default(),
                default: Default::default(),
            });

            let (_, ty_ty_generics, _) = ty_generics.split_for_impl();
            (Some(tokens), quote! { #ty_ident #ty_ty_generics }, Some(opaque))
        },

        None => {
            (None, quote! { Self::#future_name #ty_generics }, None)
        }
    };

    let associated_type = match &future_default {
        Some(_) => None,
        None => Some(quote! { type #future_name #impl_generics: #life #add_token ::core::future::Future<Output = #future_output> #where_generics; })
    };

    let tokens = quote! {
        #associated_type

        #(#attrs)*
        #constness #unsafety #abi #fn_token #ident #impl_generics (#inputs #variadic) -> #return_type #where_generics #future_default #semi_token
    };

    return (tokens, extra)
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

    let (life, _) = future_generics(inputs.iter_mut(), &mut generics);
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

fn future_generics<'a> (inputs: impl IntoIterator<Item = &'a mut FnArg>, fn_generics: &mut Generics) -> (Punctuated<TokenStream, Token![+]>, Option<Lifetime>) {
    // Reciever generics
    let mut result = None;
    for input in inputs {
        if let FnArg::Receiver(Receiver { attrs, reference, .. }) = input {            
            match reference {
                Some((x, lt @ None)) => {
                    let lifetime: Lifetime = parse_quote_spanned! { x.span => '__self__ };
                    *lt = Some(lifetime.clone());
                    result = Some(lifetime.clone());

                    fn_generics.params.insert(0, LifetimeDef {
                        attrs: attrs.clone(),
                        lifetime: lifetime.clone(),
                        colon_token: Default::default(),
                        bounds: Default::default(),
                    }.into());

                    fn_generics.make_where_clause().predicates.push(parse_quote! { Self: #lt });
                },

                Some((_, Some(lt))) => {
                    result = Some(lt.clone());
                    fn_generics.make_where_clause().predicates.push(parse_quote! { Self: #lt });
                },

                _ => {}
            }
            
            break;
        }
    }

    let tokens = fn_generics.lifetimes().map(|LifetimeDef { attrs, lifetime, .. }| 
        quote! { #(#attrs)* #lifetime }
    ).collect();

    return (tokens, result);
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