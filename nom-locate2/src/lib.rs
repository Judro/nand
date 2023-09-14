use std::fmt::{self, Debug};

use nom::{error::ParseError, IResult, Parser};
use nom_locate::{position, LocatedSpan};
pub type Input<'s> = LocatedSpan<&'s str>;

pub struct Located<'s, T>(LocatedSpan<&'s str>, T);
impl<'s, T> Located<'s, T> {
    pub fn get(&self) -> &T {
        &self.1
    }
    pub fn offset(&self) -> usize {
        self.0.location_offset()
    }
}

impl<'s, T: CharsLen> Located<'s, T> {
    pub fn span(&self) -> (usize, usize) {
        (
            self.offset(),
            self.offset() + self.get().chars_len().saturating_sub(1),
        )
    }
}

impl<'s, T: Copy> Copy for Located<'s, T> {}

impl<'s, T: Copy> Clone for Located<'s, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'s, T: PartialEq> PartialEq<T> for Located<'s, T> {
    fn eq(&self, other: &T) -> bool {
        self.get().eq(other)
    }
}

impl<'s, T> PartialEq<Located<'s, T>> for Located<'s, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0) && self.1.eq(&other.1)
    }
}

impl<'s, T: Debug> fmt::Debug for Located<'s, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Located").field(&self.1).finish()
    }
}
pub trait CharsLen {
    fn chars_len(&self) -> usize;
}

impl CharsLen for &str {
    fn chars_len(&self) -> usize {
        self.len()
    }
}

pub fn locate<'s, O, E: ParseError<Input<'s>>, F>(
    mut f: F,
) -> impl FnMut(Input<'s>) -> IResult<Input<'s>, Located<'s, O>, E>
where
    F: Parser<Input<'s>, O, E>,
    E: ParseError<Input<'s>>,
{
    move |i: Input<'s>| {
        let (i, pos) = position(i)?;
        match f.parse(i) {
            Err(e) => Err(e),
            Ok((i, o)) => Ok((i, Located(pos, o))),
        }
    }
}

pub fn get_str<'s, E: ParseError<Input<'s>>, F>(
    mut f: F,
) -> impl FnMut(Input<'s>) -> IResult<Input<'s>, &'s str, E>
where
    F: Parser<Input<'s>, Input<'s>, E>,
    E: ParseError<Input<'s>>,
{
    move |i: Input<'s>| match f.parse(i) {
        Err(e) => Err(e),
        Ok((i, o)) => Ok((i, o.into_fragment())),
    }
}

#[cfg(test)]
mod tests {
    use nom::{character::complete::alpha0, character::complete::char, sequence::delimited};

    use super::*;

    fn test_parser<'s>(input: Input<'s>) -> IResult<Input<'s>, Located<'s, &str>> {
        delimited(char('('), locate(get_str(alpha0)), char(')'))(input)
    }

    #[test]
    fn span_1() {
        let (_res, l_str) = test_parser(Input::new("(hello) ")).unwrap();

        assert_eq!(*l_str.get(), "hello");
        assert_eq!(l_str.span(), (1, 5));
    }
    #[test]
    fn span_2() {
        let (_res, l_str) = test_parser(Input::new("(h) ")).unwrap();

        assert_eq!(*l_str.get(), "h");
        assert_eq!(l_str.span(), (1, 1));
    }

    #[test]
    fn span_3() {
        let (_res, l_str) = test_parser(Input::new("() ")).unwrap();

        assert_eq!(*l_str.get(), "");
        assert_eq!(l_str.span(), (1, 1));
    }
}
