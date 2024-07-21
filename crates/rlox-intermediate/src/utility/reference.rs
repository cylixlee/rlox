use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::NonNull;

pub struct Reference<T> {
    pointer: NonNull<dyn Any>,
    _marker: PhantomData<T>,
}

impl<T: 'static> Reference<T> {
    pub unsafe fn new(pointer: *mut T) -> Self {
        Self {
            pointer: NonNull::new_unchecked(pointer as *mut dyn Any),
            _marker: PhantomData,
        }
    }

    pub unsafe fn finalize(&mut self) {
        ptr::drop_in_place(self.pointer.as_mut())
    }

    pub unsafe fn cast<U: 'static>(self) -> Reference<U> {
        Reference::<U> {
            pointer: self.pointer,
            _marker: PhantomData,
        }
    }

    pub fn downcast_ref<U: 'static>(&self) -> Option<&U> {
        unsafe { self.pointer.as_ref().downcast_ref() }
    }

    pub fn downcast_mut<U: 'static>(&mut self) -> Option<&mut U> {
        unsafe { self.pointer.as_mut().downcast_mut() }
    }
}

impl<T> Clone for Reference<T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for Reference<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.pointer.cast().as_ref() }
    }
}

impl<T> DerefMut for Reference<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.pointer.cast().as_mut() }
    }
}

impl<T, U> PartialEq<Reference<U>> for Reference<T> {
    fn eq(&self, other: &Reference<U>) -> bool {
        ptr::eq(self.pointer.as_ptr(), other.pointer.as_ptr())
    }
}

impl<T> Eq for Reference<T> {}

impl<T> Hash for Reference<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pointer.hash(state)
    }
}

impl<T> Debug for Reference<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<object: {:p}>", self.pointer)
    }
}
