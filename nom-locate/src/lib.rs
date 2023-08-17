use std::{
    fmt::{self, Debug},
    ops::{Deref, DerefMut},
};

use nom::{error::ParseError, IResult, InputLength, Offset, Parser};

pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl<T, I: Offset> Located<T, I> {
    pub fn span(&self, src: I, size: usize) -> Span {
        let offset = src.offset(&self.1);
        let start = offset.saturating_sub(size);
        let end = offset.saturating_sub(1);

        Span { start, end }
    }
    pub fn get(&self) -> &T {
        &self.0
    }
}

impl<T, I: Offset> Deref for Located<T, I> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, I: Offset> DerefMut for Located<T, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Located<&'s str, &'s str> {
    pub fn auto_span(&self, src: &'s str) -> Span {
        let offset = src.offset(self.1);
        let start = offset.saturating_sub(self.0.len());
        let end = offset.saturating_sub(1);

        Span { start, end }
    }
}

pub struct Located<T, I: Offset>(T, I);

pub fn locate<I, O, E: ParseError<I>, F>(mut f: F) -> impl FnMut(I) -> IResult<I, Located<O, I>, E>
where
    I: Clone + InputLength + Offset,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    move |i: I| match f.parse(i) {
        Err(e) => Err(e),
        Ok((i, o)) => Ok((i.clone(), Located(o, i))),
    }
}

impl<T, I: Offset> PartialEq<T> for Located<T, I>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.0.eq(other)
    }
}

impl<T, I: Offset + PartialEq> PartialEq<Located<T, I>> for Located<T, I>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0) && self.1.eq(&other.1)
    }
}

impl<T, I: Offset> fmt::Debug for Located<T, I>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Located").field("value", &self.0).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_0() {
        let src = "pub class ab";
        let class = &src[4..=8];
        let located = Located(class, &src[9..]);
        let span = located.span(src, 5);
        assert_eq!(4, span.start);
        assert_eq!(8, span.end);
    }

    #[test]
    fn auto_span_0() {
        let src = "pub class ab";
        let class = &src[4..=8];
        let located = Located(class, &src[9..]);
        let span = located.auto_span(src);
        assert_eq!(4, span.start);
        assert_eq!(8, span.end);
    }

    #[test]
    fn patial_eq_0() {
        assert_eq!(Located("a", "b"), Located("a", "b"));
        assert_ne!(Located("a", "b"), Located("a", "c"));
        assert_ne!(Located("a", "b"), Located("c", "b"));
        assert_ne!(Located("a", "b"), Located("c", "d"));
    }
}
