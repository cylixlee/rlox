use std::fmt::{Debug, Formatter};
use std::mem;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::slice::{Iter, IterMut};

use rlox_intermediate::*;

use crate::value::Value;

pub struct Stack<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    top: usize,
}

impl<T, const N: usize> Stack<T, N> {
    pub fn new() -> Self {
        Self {
            data: unsafe { mem::zeroed() },
            top: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.top
    }

    pub fn push(&mut self, element: T, span: Span) -> DiagnosableResult {
        if self.top < N {
            self.data[self.top] = MaybeUninit::new(element);
            self.top += 1;
            return Ok(());
        }
        raise!("E0006", span)
    }

    pub fn try_pop(&mut self) -> Option<T> {
        if self.top > 0 {
            self.top -= 1;
            let slot = mem::replace(&mut self.data[self.top], MaybeUninit::uninit());
            return Some(unsafe { slot.assume_init() });
        }
        None
    }

    pub fn pop(&mut self, span: Span) -> DiagnosableResult<T> {
        if let Some(value) = self.try_pop() {
            return Ok(value);
        }
        raise!("E0007", span)
    }
}

impl<T, const N: usize> Drop for Stack<T, N> {
    fn drop(&mut self) {
        while let Some(value) = self.try_pop() {
            drop(value);
        }
    }
}

impl<T, const N: usize> Deref for Stack<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        let slice: &[T; N] = unsafe { mem::transmute(&self.data) };
        &slice[0..self.top]
    }
}

impl<T, const N: usize> DerefMut for Stack<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let slice: &mut [T; N] = unsafe { mem::transmute(&mut self.data) };
        &mut slice[0..self.top]
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a Stack<T, N> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref().into_iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut Stack<T, N> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().into_iter()
    }
}

impl<const N: usize> Debug for Stack<Value, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "   ")?;
        for element in self {
            match element {
                Value::Number(number) => write!(f, "  [ {number} ]"),
                Value::Boolean(boolean) => write!(f, "  [ {boolean} ]"),
                Value::Nil => write!(f, "  [ nil ]"),
                Value::String(string) => write!(f, "  [ {} ]", &**string),
            }?
        }
        Ok(())
    }
}
