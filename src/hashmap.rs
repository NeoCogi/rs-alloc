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
use crate::*;
use crate::hash::*;


struct KeyValue<K : Hash + PartialEq, V> {
    hash    : usize,
    key     : K,
    value   : V,
}

impl<K: Hash + PartialEq, V> KeyValue<K, V> {
    pub fn is_empty(&self) -> bool { self.hash == 0 }
}

pub struct HashMap<K: Hash + PartialEq, V> {
    table   : Unique<KeyValue<K, V>>,
    capacity: usize,
    count   : usize,
}

impl<K: Hash + PartialEq, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self {
            table   : Unique::new(ptr::null_mut()),
            count   : 0,
            capacity: 0,
        }
    }

    pub fn count(&self) -> usize { self.count }

    #[inline]
    fn hash(k: &K) -> usize {
        match k.hash() {
            0 => 1,
            h => h,
        }
    }

    #[inline]
    fn next(&self, index: isize) -> isize {
        let mut ret = index - 1;
        if ret < 0 { ret += self.capacity as isize }
        ret
    }

    fn unchecked_set(&mut self, k: K, v: V) {
        let hash    = Self::hash(&k);
        let mut index   = (hash & (self.capacity - 1)) as isize;
        let entries = unsafe { core::slice::from_raw_parts_mut(self.table.get_mut_ptr(), self.capacity) };

        for _ in 0..self.capacity {
            let mut e = &mut entries[index as usize];
            if e.is_empty() {
                e.key   = k;
                e.value = v;
                e.hash  = hash;
                self.count += 1;
                return;
            }

            if hash == e.hash && k == e.key {
                e.value = v;
                return;
            }

            index = self.next(index);
        }
        panic!("uncheckedSet shouldn't reach this point");
    }

    fn new_with_cap(cap: usize) -> Self {
        Self {
            table   : Unique::new(unsafe { alloc_array_zeroed(cap) }),
            count   : 0,
            capacity: cap,
        }
    }

    fn grow(&mut self, new_cap: usize) {
        let old_entries  = unsafe { core::slice::from_raw_parts(self.table.get_ptr(), self.capacity) };
        let mut new_hm   = HashMap::<K, V>::new_with_cap(new_cap);
        for o in old_entries {
            if !o.is_empty() {
                unsafe {
                new_hm.unchecked_set(::core::ptr::read_unaligned(&o.key),
                                   ::core::ptr::read_unaligned(&o.value));
                }
            }
        }

        unsafe { free_array_ptr(self.table.get_mut_ptr(), self.capacity) };
        self.count  = 0;
        self.capacity = 0;

        *self = new_hm;
    }

    pub fn set(&mut self, k: K, v: V) {
        if 4 * self.count >= 3 * self.capacity {
            self.grow(if self.capacity == 0 { 4 } else { self.capacity * 2 });
        }
        self.unchecked_set(k, v)
    }

    pub fn exist(&self, k: K) -> bool {
        let hash = Self::hash(&k);
        let mut index   = (hash & (self.capacity - 1)) as isize;
        let entries = unsafe { core::slice::from_raw_parts(self.table.get_ptr(), self.capacity) };

        for _ in 0..self.capacity {
            let e = &entries[index as usize];
            if e.is_empty() {
                return false;
            }

            if hash == e.hash && k == e.key {
                return true;
            }

            index = self.next(index);
        }
        false
    }

    pub fn get(&self, k: K) -> Option<&V> {
        let hash = Self::hash(&k);
        let mut index   = (hash & (self.capacity - 1)) as isize;
        let entries = unsafe { core::slice::from_raw_parts(self.table.get_ptr(), self.capacity) };

        for _ in 0..self.capacity {
            let e = &entries[index as usize];
            if e.is_empty() {
                return None;
            }

            if hash == e.hash && k == e.key {
                return Some(&e.value);
            }

            index = self.next(index);
        }
        None
    }

    pub fn remove(&mut self, k: K) {
        let hash = Self::hash(&k);
        let mut index   = (hash & (self.capacity - 1)) as isize;
        let entries = unsafe { core::slice::from_raw_parts_mut(self.table.get_mut_ptr(), self.capacity) };

        for _ in 0..self.capacity {
            let e = &entries[index as usize];
            if e.is_empty() {
                return;
            }

            if hash == e.hash && k == e.key {
                self.count -= 1;
                break;
            }

            index = self.next(index);
        }

        loop {
            let empty_index = index;
            let mut original_index;
            loop {
                index = self.next(index);
                let s = &entries[index as usize];
                if s.is_empty() {
                    entries[empty_index as usize].hash = 0;
                    unsafe { ::core::ptr::read_unaligned(&entries[empty_index as usize]) };  // drop it!
                    return;
                }

                original_index   = (s.hash & (self.capacity - 1)) as isize;

                if ! ((index <= original_index && original_index < empty_index)
                    || (original_index < empty_index && empty_index < index)
                    || (empty_index < index && index <= original_index)) {
                    break;
                }
            }

            entries[empty_index as usize] = unsafe { ::core::ptr::read_unaligned(&entries[index as usize]) };
            entries[index as usize].hash = 0;
        }
    }
}

impl<K : Hash + PartialEq, V> Drop for HashMap<K, V> {
    fn drop(&mut self) {
            if self.capacity > 0 {
            let arr      = unsafe { core::slice::from_raw_parts_mut(self.table.get_mut_ptr(), self.capacity) };
            for kv in arr {
                if !kv.is_empty() {
                    unsafe { ptr::drop_in_place(&kv.key as *const K as *mut K) };
                    unsafe { ptr::drop_in_place(&kv.value as *const V as *mut V) };
                }
            }
            unsafe { free_array_ptr(self.table.get_mut_ptr(), self.capacity) }
        }
    }
}

impl Hash for i32 {
    fn hash(&self) -> usize { *self as usize }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::vec::*;

    #[test]
    fn test_insert() {
        let mut hm = HashMap::<i32, i32>::new();
        for i in 0..100 {
            hm.set(i, i * 2);
        }

        for i in 0..100 {
            let ret = hm.get(i);
            assert!(ret.is_some());
            match ret {
                Some(o) => assert!(*o == i * 2),
                None => assert!(false)
            }
        }
    }

    #[test]
    fn test_remove() {
        let mut hm = HashMap::<i32, i32>::new();
        for i in 0..100 {
            hm.set(i, i * 2);
        }

        for i in 45..55 {
            hm.remove(i);
            assert!(hm.exist(i) == false);
        }

        for i in 0..45 {
            let ret = hm.get(i);
            assert!(ret.is_some());
            match ret {
                Some(o) => assert!(*o == i * 2),
                None => assert!(false)
            }
        }

        for i in 55..100 {
            let ret = hm.get(i);
            assert!(ret.is_some());
            match ret {
                Some(o) => assert!(*o == i * 2),
                None => assert!(false)
            }
        }

        for i in 45..55 {
            assert!(hm.exist(i) == false);
        }

        assert!(hm.count() == 90);
    }

    #[test]
    fn test_vec_insert() {
        let mut hm = HashMap::<i32, Vec<i32>>::new();
        for i in 0..100 {
            let mut v = Vec::new();
            for j in 0..i * 2 {
                v.push(j);
            }
            hm.set(i, v);
        }

        for i in 0..100 {
            let ret = hm.get(i);
            assert!(ret.is_some());
            match ret {
                Some(o) => {
                    for j in 0..i*2 {
                        assert!((*o)[j as usize] == j)
                    }
                }
                None => assert!(false)
            }
        }
    }
/*
    #[test]
    fn test_vec_remove() {
        let mut hm = HashMap::<i32, Vec<i32>>::new();
        for i in 0..100 {
            let mut v = Vec::new();
            for j in 0..i * 2 {
                v.push(j);
            }
            hm.set(i, v);
        }

        for i in 45..55 {
            hm.remove(i);
            assert!(hm.exist(i) == false);
        }

        for i in 0..45 {
            let ret = hm.get(i);
            assert!(ret.is_some());
            match ret {
                Some(o) => {
                    for j in 0..i*2 {
                        assert!((*o)[j as usize] == j)
                    }
                },
                None => assert!(false)
            }
        }

        for i in 55..100 {
            let ret = hm.get(i);
            assert!(ret.is_some());
            match ret {
                Some(o) => {
                    for j in 0..i*2 {
                        assert!((*o)[j as usize] == j)
                    }
                },
                None => assert!(false)
            }
        }

        for i in 45..55 {
            assert!(hm.exist(i) == false);
        }

        assert!(hm.count() == 90);
    }
*/
}
