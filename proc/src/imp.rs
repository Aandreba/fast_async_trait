use syn::{*, parse::{Parse, ParseStream}};

pub struct AsyncItemImpl {
    pub attrs: Vec<Attribute>,
    pub defaultness: Option<Token![default]>,
    pub unsafety: Option<Token![unsafe]>,
    pub impl_token: Token![impl],
    pub generics: Generics,
    pub trait_: Option<(Option<Token![!]>, Path, Token![for])>,
    pub self_ty: Box<Type>,
    pub brace_token: syn::token::Brace,
    pub items: Vec<ImplItem>,
}

impl Parse for AsyncItemImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        let allow_verbatim_impl = false;
        parse_impl(input, allow_verbatim_impl).map(Option::unwrap)
    }
}

fn parse_impl(input: ParseStream, allow_verbatim_impl: bool) -> Result<Option<AsyncItemImpl>> {
    let mut attrs = input.call(Attribute::parse_outer)?;
    let has_visibility = allow_verbatim_impl && !matches!(input.parse::<Visibility>()?, Visibility::Inherited);
    let defaultness: Option<Token![default]> = input.parse()?;
    let unsafety: Option<Token![unsafe]> = input.parse()?;
    let impl_token: Token![impl] = input.parse()?;

    let has_generics = input.peek(Token![<])
        && (input.peek2(Token![>])
            || input.peek2(Token![#])
            || (input.peek2(Ident) || input.peek2(Lifetime))
                && (input.peek3(Token![:])
                    || input.peek3(Token![,])
                    || input.peek3(Token![>])
                    || input.peek3(Token![=]))
            || input.peek2(Token![const]));
    let mut generics: Generics = if has_generics {
        input.parse()?
    } else {
        Generics::default()
    };

    let is_const_impl = allow_verbatim_impl
        && (input.peek(Token![const]) || input.peek(Token![?]) && input.peek2(Token![const]));
    if is_const_impl {
        input.parse::<Option<Token![?]>>()?;
        input.parse::<Token![const]>()?;
    }

    //let begin = input.fork();
    let polarity = if input.peek(Token![!]) && !input.peek2(token::Brace) {
        Some(input.parse::<Token![!]>()?)
    } else {
        None
    };

    #[cfg(not(feature = "printing"))]
    let first_ty_span = input.span();
    let mut first_ty: Type = input.parse()?;
    let self_ty: Type;
    let trait_;

    let is_impl_for = input.peek(Token![for]);
    if is_impl_for {
        let for_token: Token![for] = input.parse()?;
        let mut first_ty_ref = &first_ty;
        while let Type::Group(ty) = first_ty_ref {
            first_ty_ref = &ty.elem;
        }
        if let Type::Path(TypePath { qself: None, .. }) = first_ty_ref {
            while let Type::Group(ty) = first_ty {
                first_ty = *ty.elem;
            }
            if let Type::Path(TypePath { qself: None, path }) = first_ty {
                trait_ = Some((polarity, path, for_token));
            } else {
                unreachable!();
            }
        } else if !allow_verbatim_impl {
            #[cfg(feature = "printing")]
            return Err(Error::new_spanned(first_ty_ref, "expected trait path"));
            #[cfg(not(feature = "printing"))]
            return Err(Error::new(first_ty_span, "expected trait path"));
        } else {
            trait_ = None;
        }
        self_ty = input.parse()?;
    } else {
        trait_ = None;
        self_ty = if polarity.is_none() {
            first_ty
        } else {
            todo!()
            //Type::Verbatim(verbatim::between(begin, input))
        };
    }

    generics.where_clause = input.parse()?;

    let content;
    let brace_token = braced!(content in input);
    attrs.append(&mut Attribute::parse_inner(&content)?);

    let mut items = Vec::new();
    while !content.is_empty() {
        items.push(content.parse()?);
    }

    if has_visibility || is_const_impl || is_impl_for && trait_.is_none() {
        Ok(None)
    } else {
        Ok(Some(AsyncItemImpl {
            attrs,
            defaultness,
            unsafety,
            impl_token,
            generics,
            trait_,
            self_ty: Box::new(self_ty),
            brace_token,
            items,
        }))
    }
}