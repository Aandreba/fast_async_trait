use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{*, punctuated::{Punctuated}, parse::{Parse, ParseStream}};

#[derive(Parse)]
pub struct AsyncTraitItemMethod {
    #[call(Attribute::parse_outer)]
    pub attrs: Vec<Attribute>,
    pub sig: Signature,
    #[peek(syn::token::Brace)]
    pub default: Option<Block>,
    pub semi_token: Option<Token![;]>,
}

#[non_exhaustive]
pub enum AsyncTraitItem {
    Const(TraitItemConst),
    Method(AsyncTraitItemMethod),
    Type(TraitItemType),
    Macro(TraitItemMacro),
    #[allow(unused)]
    Verbatim(TokenStream),
}

pub struct AsyncTraitDef {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub unsafety: Option<Token![unsafe]>,
    pub auto_token: Option<Token![auto]>,
    pub trait_token: Token![trait],
    pub ident: Ident,
    pub generics: Generics,
    pub colon_token: Option<Token![:]>,
    pub supertraits: Punctuated<TypeParamBound, Token![+]>,
    pub brace_token: syn::token::Brace,
    pub items: Vec<AsyncTraitItem>,
}

impl ToTokens for AsyncTraitItemMethod {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        fn is_outer(attr: &&Attribute) -> bool {
            match attr.style {
                AttrStyle::Outer => true,
                AttrStyle::Inner(_) => false,
            }
        }

        tokens.append_all(self.attrs.iter().filter(is_outer));
        self.sig.to_tokens(tokens);
        self.default.to_tokens(tokens);
        self.semi_token.to_tokens(tokens);
    }
}

impl ToTokens for AsyncTraitItem {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Const(x) => x.to_tokens(tokens),
            Self::Macro(x) => x.to_tokens(tokens),
            Self::Method(x) => x.to_tokens(tokens),
            Self::Type(x) => x.to_tokens(tokens),
            Self::Verbatim(x) => x.to_tokens(tokens)
        }
    }
}

impl Parse for AsyncTraitDef {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let outer_attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let unsafety: Option<Token![unsafe]> = input.parse()?;
        let auto_token: Option<Token![auto]> = input.parse()?;
        let trait_token: Token![trait] = input.parse()?;
        let ident: Ident = input.parse()?;
        let generics: Generics = input.parse()?;
        return parse_rest_of_trait(
            input,
            outer_attrs,
            vis,
            unsafety,
            auto_token,
            trait_token,
            ident,
            generics,
        );

        fn parse_rest_of_trait(
            input: parse::ParseStream,
            mut attrs: Vec<Attribute>,
            vis: Visibility,
            unsafety: Option<Token![unsafe]>,
            auto_token: Option<Token![auto]>,
            trait_token: Token![trait],
            ident: Ident,
            mut generics: Generics,
        ) -> Result<AsyncTraitDef> {
            let colon_token: Option<Token![:]> = input.parse()?;
    
            let mut supertraits = Punctuated::new();
            if colon_token.is_some() {
                loop {
                    if input.peek(Token![where]) || input.peek(token::Brace) {
                        break;
                    }
                    supertraits.push_value(input.parse()?);
                    if input.peek(Token![where]) || input.peek(token::Brace) {
                        break;
                    }
                    supertraits.push_punct(input.parse()?);
                }
            }
    
            generics.where_clause = input.parse()?;
    
            let content;
            let brace_token = braced!(content in input);
            attrs.append(&mut Attribute::parse_inner(&content)?);
            let mut items = Vec::new();
            while !content.is_empty() {
                items.push(content.parse()?);
            }
    
            Ok(AsyncTraitDef {
                attrs,
                vis,
                unsafety,
                auto_token,
                trait_token,
                ident,
                generics,
                colon_token,
                supertraits,
                brace_token,
                items,
            })
        }
    }
}

impl Parse for AsyncTraitItem {
    fn parse(input: ParseStream) -> Result<Self> {
        //TraitItem
        //let begin = input.fork();
        let mut attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let defaultness: Option<Token![default]> = input.parse()?;
        let ahead = input.fork();

        let lookahead = ahead.lookahead1();
        let mut item = if lookahead.peek(Token![fn]) || peek_signature(&ahead) {
            input.parse().map(AsyncTraitItem::Method)
        } else if lookahead.peek(Token![const]) {
            ahead.parse::<Token![const]>()?;
            let lookahead = ahead.lookahead1();
            if lookahead.peek(Ident) || lookahead.peek(Token![_]) {
                input.parse().map(AsyncTraitItem::Const)
            } else if lookahead.peek(Token![async])
                || lookahead.peek(Token![unsafe])
                || lookahead.peek(Token![extern])
                || lookahead.peek(Token![fn])
            {
                input.parse().map(AsyncTraitItem::Method)
            } else {
                Err(lookahead.error())
            }
        } else if lookahead.peek(Token![type]) {
            input.parse().map(AsyncTraitItem::Type)
        } else if lookahead.peek(Ident)
            || lookahead.peek(Token![self])
            || lookahead.peek(Token![super])
            || lookahead.peek(Token![crate])
            || lookahead.peek(Token![::])
        {
            input.parse().map(AsyncTraitItem::Macro)
        } else {
            Err(lookahead.error())
        }?;

        match (vis, defaultness) {
            (Visibility::Inherited, None) => {}
            _ => todo!()
            //_ => return Ok(AsyncTraitItem::Verbatim(verbatim_between(begin, input))),
        }

        let item_attrs = match &mut item {
            AsyncTraitItem::Const(item) => &mut item.attrs,
            AsyncTraitItem::Method(item) => &mut item.attrs,
            AsyncTraitItem::Type(item) => &mut item.attrs,
            AsyncTraitItem::Macro(item) => &mut item.attrs,
            AsyncTraitItem::Verbatim(_) => unreachable!(),

            #[cfg(syn_no_non_exhaustive)]
            _ => unreachable!(),
        };
        attrs.append(item_attrs);
        *item_attrs = attrs;
        Ok(item)
    }
}

fn peek_signature(input: ParseStream) -> bool {
    let fork = input.fork();
    fork.parse::<Option<Token![const]>>().is_ok()
        && fork.parse::<Option<Token![async]>>().is_ok()
        && fork.parse::<Option<Token![unsafe]>>().is_ok()
        && fork.parse::<Option<Abi>>().is_ok()
        && fork.peek(Token![fn])
}