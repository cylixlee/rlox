use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut, Range};

pub type Span = Range<usize>;

pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: impl Into<Span>) -> Self {
        Self {
            value,
            span: span.into(),
        }
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> Clone for Spanned<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            span: self.span.clone(),
        }
    }
}

impl<T> Debug for Spanned<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
