//
// Copyright 2020-Present (c) Raja Lehtihet & Wael El Oraiby
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
// may be used to endorse or promote products derived from this software without
// specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.
//

use core::*;
use core::ops::*;
use core::slice::*;
use crate::*;


#[repr(C)]
pub struct Vec<T> {
    elements    : *mut T,
    count       : usize,
    capacity    : usize,
}

impl<T> Vec<T> {
    pub fn with_capacity(c: usize) -> Self {
        if c == 0 { Self::new() }
        else {
            Self {
                elements: unsafe { alloc_array(c) },
                count   : 0,
                capacity: c,
            }
        }
    }

    pub fn new() -> Self {
        Self {
            elements: ptr::null_mut(),
            count   : 0,
            capacity: 0,
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] { unsafe { core::slice::from_raw_parts(self.elements, self.count) } }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] { unsafe { core::slice::from_raw_parts_mut(self.elements, self.count) } }

    pub fn len(&self) -> usize { self.count }

    pub fn push(&mut self, t: T) {
        if self.count >= self.capacity {
            let new_size    = if self.capacity == 0 { 16 } else { self.capacity * 2 };
            let new_ptr     = unsafe { alloc_array::<T>(new_size) };
            let old_ptr     = self.elements;

            for i in 0..self.count {
                let v = unsafe { old_ptr.offset(i as isize).read() };    // v = old[i];
                unsafe { new_ptr.offset(i as isize).write(v) };          // new[i] = v;
            }
            unsafe { free_array_ptr(self.elements, self.capacity) };
            self.elements   = new_ptr;
            self.capacity   = new_size;
        }

        unsafe { self.elements.offset(self.count as isize).write(t) };
        self.count += 1
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 { None }
        else {
            let nc = self.count - 1;
            let v = unsafe { ptr::read(self.get_unchecked(nc) as *const _) };
            self.count -= 1;
            Some(v)
        }
    }

    #[inline]
    pub fn get_unchecked(&self, idx: usize) -> &T {
        let arr      = unsafe { core::slice::from_raw_parts(self.elements, self.count) };
        &arr[idx]
    }

    #[inline]
    pub fn get_unchecked_mut(&mut self, idx: usize) -> &mut T {
        let arr      = unsafe { core::slice::from_raw_parts_mut(self.elements, self.count) };
        &mut arr[idx]
    }

    fn drop_elements(&mut self) {
        let arr      = unsafe { core::slice::from_raw_parts_mut(self.elements, self.count) };
        for i in 0..self.count {
            unsafe { ptr::drop_in_place(&arr[i] as *const T as *mut T) };
        }
    }

    pub fn to_iter<'a>(&self) -> ::core::slice::Iter<'a, T> {
        let arr      = unsafe { core::slice::from_raw_parts(self.elements, self.count) };
        arr.into_iter()
    }

    pub fn last(&self) -> Option<&T> {
        if self.count == 0 {
            None
        } else {
            Some(&self[self.count - 1])
        }
    }

    pub fn capacity(&self) -> usize { self.capacity }

    pub fn iter(&self) -> slice::Iter<T> {
        self.as_slice().into_iter()
    }

    pub fn iter_mut(&mut self) -> slice::IterMut<T> {
        self.as_mut_slice().into_iter()
    }

    pub fn from_raw_parts(ptr: *mut T, len: usize, cap: usize) -> Self {
        Self { elements: ptr, count: len, capacity: cap }
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> slice::Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vec<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> slice::IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<A> iter::FromIterator<A> for Vec<A> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        let mut v = Vec::new();
        let mut it = iter.into_iter();
        loop {
            match it.next() {
                Some(r) => v.push(r),
                None => break,
            }
        }
        v
    }
}

pub trait VecAppend<E: Copy> {
    fn append(&mut self, arr: &[E]);
}

impl<T : Copy> VecAppend<T> for Vec<T> {
    fn append(&mut self, arr: &[T]) {
        // TODO: optimize this
        for e in arr {
            self.push(e.clone());
        }
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for Vec<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.as_slice(), index)
    }
}

impl<T, I: SliceIndex<[T]>> IndexMut<I> for Vec<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(self.as_mut_slice(), index)
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        self.drop_elements();
        unsafe { free_array_ptr(self.elements, self.capacity) }
    }
}

impl<T : Clone> Clone for Vec<T> {
    fn clone(&self) -> Self {
        let mut c = Vec::<T>::new();
        for i in 0..self.count {
            let v = self.get_unchecked(i);
            c.push(v.clone());
        }
        c
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destructor() {
        let mut v = Vec::<Vec<i32>>::new();
        for i in 0..100 {
            let  mut vj = Vec::<i32>::new();
            for j in 0..100 {
                vj.push(j * i);
            }
            v.push(vj);
        }
    }

    #[test]
    fn test_iter() {
        let mut v = Vec::new();
        for i in 0..4 {
            v.push(i);
            assert!(v[i] == i);
        }

        let mut counter = 0;
        for i in v.to_iter() {
            if *i != counter { panic!("invalid {} != {}", i, counter) }
            counter += 1;
        }
    }
    #[test]
    fn test_pop_destructor() {
        let mut v = Vec::<Vec<i32>>::new();
        for i in 0..100 {
            let  mut vj = Vec::<i32>::new();
            for j in 0..100 {
                vj.push(j * i);
            }
            v.push(vj);
        }

        assert!(v.len() == 100);
        for _ in 0..100 {
            v.pop();
        }
        assert!(v.len() == 0);
    }

    #[test]
    fn test_pop_destructor_push() {
        let mut v = Vec::<Vec<i32>>::new();
        for i in 0..100 {
            let  mut vj = Vec::<i32>::new();
            for j in 0..100 {
                vj.push(j * i);
            }
            v.push(vj);
        }

        for _ in 0..100 {
            v.pop();
        }

        assert!(v.len() == 0);

        for i in 0..100 {
            let  mut vj = Vec::<i32>::new();
            for j in 0..100 {
                vj.push(j * i);
            }
            v.push(vj);
        }

        assert!(v.len() == 100);
    }
}
