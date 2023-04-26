use std::marker::PhantomData;

use combine::error::Tracked;
use combine::stream::easy::{Error, Errors, Info};
use combine::{satisfy, ConsumedResult, Parser, StreamOnce};

use crate::position::Pos;
use crate::tokenizer::{Kind, Token, TokenStream};

use super::common::Text;

#[derive(Debug, Clone)]
pub struct TokenMatch<'a> {
    kind: Kind,
    phantom: PhantomData<&'a u8>,
}

#[derive(Debug, Clone)]
pub struct NameMatch<'a, T>
where
    T: Text<'a>,
{
    phantom: PhantomData<&'a T>,
}

#[derive(Debug, Clone)]
pub struct Value<'a> {
    kind: Kind,
    value: &'static str,
    phantom: PhantomData<&'a u8>,
}

pub fn kind<'x>(kind: Kind) -> TokenMatch<'x> {
    TokenMatch {
        kind,
        phantom: PhantomData,
    }
}

pub fn name<'a, T>() -> NameMatch<'a, T>
where
    T: Text<'a>,
{
    NameMatch {
        phantom: PhantomData,
    }
}

impl<'a> Parser for TokenMatch<'a> {
    type Input = TokenStream<'a>;
    type Output = Token<'a>;
    type PartialState = ();

    #[inline]
    fn parse_lazy(
        &mut self,
        input: &mut Self::Input,
    ) -> ConsumedResult<Self::Output, Self::Input> {
        satisfy(|c: Token<'a>| c.kind == self.kind).parse_lazy(input)
    }

    fn add_error(&mut self, error: &mut Tracked<Errors<Token<'a>, Token<'a>, Pos>>) {
        error
            .error
            .add_error(Error::Expected(Info::Owned(format!("{:?}", self.kind))));
    }
}

pub fn punct<'s>(value: &'static str) -> Value<'s> {
    Value {
        kind: Kind::Punctuator,
        value,
        phantom: PhantomData,
    }
}

pub fn ident<'s>(value: &'static str) -> Value<'s> {
    Value {
        kind: Kind::Name,
        value,
        phantom: PhantomData,
    }
}

impl<'a> Parser for Value<'a> {
    type Input = TokenStream<'a>;
    type Output = Token<'a>;
    type PartialState = ();

    #[inline]
    fn parse_lazy(
        &mut self,
        input: &mut Self::Input,
    ) -> ConsumedResult<Self::Output, Self::Input> {
        satisfy(|c: Token<'a>| c.kind == self.kind && c.value == self.value)
            .parse_lazy(input)
    }

    fn add_error(&mut self, error: &mut Tracked<<Self::Input as StreamOnce>::Error>) {
        error
            .error
            .add_error(Error::Expected(Info::Borrowed(self.value)));
    }
}

impl<'a, S> Parser for NameMatch<'a, S>
where
    S: Text<'a>,
{
    type Input = TokenStream<'a>;
    type Output = S::Value;
    type PartialState = ();

    #[inline]
    fn parse_lazy(
        &mut self,
        input: &mut Self::Input,
    ) -> ConsumedResult<Self::Output, Self::Input> {
        satisfy(|c: Token<'a>| c.kind == Kind::Name)
            .map(|t: Token<'a>| -> S::Value { S::Value::from(t.value) })
            .parse_lazy(input)
    }

    fn add_error(&mut self, error: &mut Tracked<Errors<Token<'a>, Token<'a>, Pos>>) {
        error
            .error
            .add_error(Error::Expected(Info::Borrowed("Name")));
    }
}
