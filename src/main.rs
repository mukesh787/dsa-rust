
use std::ptr;
use std::{ptr::NonNull};
use std::mem::ManuallyDrop;

use std::alloc::{self, Layout};

pub struct Vec<T> {
    ptr:NonNull<T>,
    len: usize,         
    cap: usize,
}

impl<T> Vec<T>{
    pub fn new()->Vec<T>{
        Vec { ptr: NonNull::dangling(), len:0, cap: 0 }
    }

    pub fn grow(&mut self){
        let (new_cap,new_layout)=if self.cap==0{
            (1,Layout::array::<T>(1).unwrap())
        }else{
            let new_cap=2*self.cap;
            let new_layout=Layout::array::<T>(new_cap).unwrap();
            (new_cap,new_layout)
        };
        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        self.ptr=match NonNull::new(new_ptr as *mut T){
            Some(p)=>p,
            None=>alloc::handle_alloc_error(new_layout),
        };
        self.cap=new_cap;
    }

    pub fn push(&mut self,elem:T){
        if self.len==self.cap{
            self.grow();
        }
        unsafe{
            ptr::write(self.ptr.as_ptr().add(self.len),elem );
        }
        self.len=self.len+1;
    }

    pub fn pop(&mut self)->Option<T>{
        if self.len==0{
            None
        }else{
            self.len-=self.len;
        unsafe{
            Some(ptr::read(self.ptr.as_ptr().add(self.len)))
           }
        }
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        // Note: `<=` because it's valid to insert after everything
        // which would be equivalent to push.
        assert!(index <= self.len, "index out of bounds");
        if self.len == self.cap { self.grow(); }
    
        unsafe {
            // ptr::copy(src, dest, len): "copy from src to dest len elems"
            ptr::copy(
                self.ptr.as_ptr().add(index),
                self.ptr.as_ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr.as_ptr().add(index), elem);
        }
    
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");
        unsafe {
            let elem = ptr::read(self.ptr.as_ptr().add(index));
            ptr::copy(
                self.ptr.as_ptr().add(index + 1),
                self.ptr.as_ptr().add(index),
                self.len - index - 1,
            );
            self.len -= 1;
            elem
        }
    }
}

impl <T> Drop for Vec<T>{
    fn drop(&mut self) {
        if self.cap!=0{
            let layout=Layout::array::<T>(self.cap).unwrap();
            unsafe{
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout)
            }
        }
    }
}

impl <T> std::ops::Deref for Vec<T>{
    type Target=[T];
    fn deref(&self)->&[T]{
        unsafe{
            std::slice::from_raw_parts(self.ptr.as_ptr(),self.len)
        }
    }
}

impl <T> std::ops::DerefMut for Vec<T>{
    fn deref_mut(&mut self)->&mut [T]{
        unsafe{
            std::slice::from_raw_parts_mut(self.ptr.as_ptr(),self.len)
        }
    }
}

pub struct IntoIter<T> {
    buf: NonNull<T>,
    cap: usize,
    start: *const T,
    end: *const T,
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> {
        // Make sure not to drop Vec since that would free the buffer
        let vec = ManuallyDrop::new(self);

        // Can't destructure Vec since it's Drop
        let ptr = vec.ptr;
        let cap = vec.cap;
        let len = vec.len;

        IntoIter {
            buf: ptr,
            cap,
            start: ptr.as_ptr(),
            end: if cap == 0 {
                // can't offset off this pointer, it's not allocated!
                ptr.as_ptr()
            } else {
                unsafe { ptr.as_ptr().add(len) }
            },
        }
    }
}

impl <T> Iterator for IntoIter<T>{
    type Item=T;
    fn next(&mut self)->Option<T>{
        if self.start==self.end{
            None
        }else{
            unsafe{
                let elem=ptr::read(self.start);
                self.start=self.start.add(1);
                Some(elem)
            }
        }
    }
}

impl <T> DoubleEndedIterator for IntoIter<T>{
    fn next_back(&mut self)->Option<T>{
        if self.start==self.end{
            None
        }else{
            unsafe{
                self.end=self.end.sub(1);
                Some(ptr::read(self.end))
            }
        }
    }
}

impl <T> Drop for IntoIter<T>{
    fn drop(&mut self){
        if self.cap!=0{
            let elem_size=std::mem::size_of::<T>();
            let layout=Layout::from_size_align(self.cap*elem_size,elem_size).unwrap();
            unsafe{
                alloc::dealloc(self.buf.as_ptr() as *mut u8,layout)
            }
        }
    }
}
#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_vec() {
        dbg!(std::mem::size_of::<Vec<i8>>());
        dbg!(std::mem::size_of::<Vec<i64>>());
        dbg!(std::mem::align_of::<Vec<i8>>());     
        //maximum of all the alignment size of  all the
        //  fields is the align_size of the struct/block
        // pub struct Vec<T> {
        //     ptr:NonNull<T>,
        //     len: u128,    // 16 bytes -it is the maximum of all the fields here     
        //     cap: u32,
        // }
        // size_of =32 bytes here in this above example. because due to alignment
        // boundary of 16 bytes. 16+8+4=28 bytes. 4 bytes padding is added to make it 32 bytes
        // align_of = 16 bytes
        // so first alignment is fixed and according to alignemnt then size is calculated
        dbg!(std::mem::align_of::<Vec<i64>>());
        assert!(false);
    }
}