extern crate num;
extern crate alloc;
extern crate rgsl;

use self::num::NumCast;
use self::num::Float;
use std::mem;

use self::alloc::heap;
use std::mem::{align_of, transmute};
use std::intrinsics;
use std::raw::Slice;
use std::i32;
use std::f32;
use std::f64;
use std::slice;
use std::marker;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops;
use std::ptr::{Unique, self};

fn get_values_as_type_t<T>(start: T, stop: T, len: usize) -> (T, T, T)
    where T: Float {
    let zero: T = num::cast(0).unwrap();
    let len_t: T = num::cast(len).unwrap();
    let one: T = num::cast(1).unwrap();
    let diff = stop - start;
    let dx = diff/(len_t-one);
    return (one, zero, dx)
}

// you can't do std::<T>::consts::PI. This is a workaround by Shepmaster (needed e.g. for sinc())
// https://github.com/rust-lang/rfcs/pull/1062
// http://stackoverflow.com/questions/32763783/how-to-access-numeric-constants-using-the-float-trait

pub trait FloatConst {
    fn pi() -> Self;
}

impl FloatConst for f32 {
    fn pi() -> Self { f32::consts::PI }
}

impl FloatConst for f64 {
    fn pi() -> Self { f64::consts::PI }
}


pub fn linspace_vec<'a, T: 'a>(start: T, stop: T, len: usize) ->
Vec<T>
    where T: Float {

    let (one, zero, dx) = get_values_as_type_t::<T>(start, stop, len);

    // let mut v = Vec::<T>::with_capacity(len);
    //
    // for i in 0..len {
    //     v.push(zero);
    // }

    let mut v = vec![zero; len];

    let mut c = zero;

    //**** SLOW ****
    // for x in v.iter_mut() {
    //     *x = start + c*dx;
    //     c = c + one;
    // }

    //**** FAST ****
    let ptr: *mut T = v.as_mut_ptr();
    unsafe {
        for ii in 0..len {
            let x = ptr.offset((ii as isize));
            *x = start + c*dx;
            c = c + one;
        }
    }

    return v
}


pub fn linspace_vec2boxed_slice<'a, T: 'a>(start: T, stop: T, len: usize) -> Box<[T]>
    where T: Float {
    // get 0, 1 and the increment dx as T
    let (one, zero, dx) = get_values_as_type_t::<T>(start, stop, len);
    let mut v = vec![zero; len].into_boxed_slice();
    let mut c = zero;
    let ptr: *mut T = v.as_mut_ptr();
    unsafe {
        for ii in 0..len {
            let x = ptr.offset((ii as isize));
            *x = start + c*dx;
            c = c + one;
        }
    }

    v
}

pub fn make_arr_unsafe<'a, T>(len: usize) -> &'a mut [T] {

    let size = len * mem::size_of::<T>();

    unsafe {
        let ptr = heap::allocate(size, align_of::<T>());
        intrinsics::volatile_set_memory(ptr, 0, size);
        let slice = slice::from_raw_parts_mut(ptr as *mut T, len);
        return slice;
    }
}

pub fn linspace_slice<'a, T: 'a>(start: T, stop: T, len: usize) -> &'a [T]
    where T: Float {

    let (one, zero, dx) = get_values_as_type_t::<T>(start, stop, len);

    let size = len * mem::size_of::<T>();

    unsafe {
        let ptr = heap::allocate(size, align_of::<T>());
        let bx = slice::from_raw_parts_mut(ptr as *mut T, len);

        let mut c = zero;

        for x in bx.iter_mut() {
            *x = start + c*dx;
            c = c + one;
        }

        return bx
    }
}



pub fn linspace_slice_unchecked<'a, T: 'a>(start: T, stop: T, len: usize) -> &'a [T]
    where T: Float {

    let (one, zero, dx) = get_values_as_type_t::<T>(start, stop, len);

    let size = len * mem::size_of::<T>();

    unsafe {
        let ptr = heap::allocate(size, align_of::<T>());
        let bx = slice::from_raw_parts_mut(ptr as *mut T, len);

        let mut c = zero;

        for ii in 0..len {
            let x = bx.get_unchecked_mut(ii);
            *x = start + c*dx;
            c = c + one;
        }

        return bx
    }
}

pub fn linspace_ptr<'a, T: 'a>(start: T, stop: T, len: usize) -> *mut T
    where T: Float {

let (one, zero, dx) = get_values_as_type_t::<T>(start, stop, len);

    let size = len * mem::size_of::<T>();

    unsafe {
        let ptr = heap::allocate(size, align_of::<T>()) as *mut T;

        let mut c = zero;

        for ii in 0..len {
            let x = ptr.offset((ii as isize));
            *x = start + c*dx;
            c = c + one;
        }

        return ptr as *mut T
    }
}

// Similar to IntermediateBox
pub struct HeapSlice<T> {
    // ptr: *mut T,
    ptr: Unique<T>,
    length: usize,
    // marker: marker::PhantomData<*mut T>,
}

fn oom() {
    ::std::process::exit(-9999);
}

// #![feature(alloc, heap_api)]

impl<T> HeapSlice<T> {
    fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "We're not ready to handle ZSTs");
        unsafe {
            // need to cast EMPTY to the actual ptr type we want, let
            // inference handle it.
            HeapSlice { ptr: Unique::new(heap::EMPTY as *mut _), length: 0 }
        }
    }

    fn allocate(&mut self, newlength: usize) {
        unsafe {
            let typesize = mem::size_of::<T>();
            let align = mem::align_of::<T>();
            let size = newlength * typesize;

            if self.length == 0 {
                let ptr = heap::allocate(size, align);
                // If allocate fails, we'll get `null` back
                if ptr.is_null() { oom(); }
                self.ptr = Unique::new(ptr as *mut _);
                self.length = newlength;
            } else {
                panic!("already allocated ?")
            }
        }
    }

}

// // Similar to an make_place for IntermediateBox
// fn alloc_heapslice<T>(length: usize) -> HeapSlice<T> {
//     let typesize = mem::size_of::<T>();
//     let size = length * typesize;
//     let align = mem::align_of::<T>();
//
//     let p = if typesize == 0 || length == 0 {
//         heap::EMPTY as *mut T
//     } else {
//         let p = unsafe {
//             heap::allocate(size, align) as *mut T
//         };
//         if p.is_null() {
//             panic!("HeapSlice make_place allocation failure.");
//         }
//         p
//     };
//
//     HeapSlice { ptr: p, length: length, typesize: typesize, size: size, align: align }
// }

impl<T> Drop for HeapSlice<T> {
    fn drop(&mut self) {
        if self.length!= 0 {
            let align = mem::align_of::<T>();
            let elem_size = mem::size_of::<T>();
            let size = elem_size * self.length;
            unsafe {
                heap::deallocate(*self.ptr as *mut u8, size, align)
            }
        }
    }
}


impl<T> Deref for HeapSlice<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(*self.ptr, self.length)
        }
    }
}

impl<T> DerefMut for HeapSlice<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(*self.ptr, self.length)
        }
    }
}

impl<T> ops::Mul<T> for HeapSlice<T> where T: Float {
    type Output = HeapSlice<T>;
    fn mul(self, f: T) -> HeapSlice<T> {
        let mut fb: HeapSlice<T> = HeapSlice::<T>::new();
        fb.allocate(self.length);
        for (xout,xin) in &mut fb.iter_mut().zip(self.iter()) {
            *xout = f*(*xin);
        }
        fb
    }
}

impl ops::Mul<i32> for HeapSlice<f64>  {
    type Output = HeapSlice<f64>;
    fn mul(self, f: i32) -> HeapSlice<f64> {
        let mut fb: HeapSlice<f64> = HeapSlice::<f64>::new();
        fb.allocate(self.length);
        for (xout,xin) in &mut fb.iter_mut().zip(self.iter()) {
            *xout = (f as f64)*(*xin);
        }
        fb
    }
}


impl<T> HeapSlice<T> where T: Float {
    pub fn sin(&self) -> HeapSlice<T> {
        let mut fb: HeapSlice<T> = HeapSlice::<T>::new();
        fb.allocate(self.length);
        for (xout,xin) in &mut fb.iter_mut().zip(self.iter()) {
            *xout = (*xin).sin();
        }
        fb
    }
}

impl<T> HeapSlice<T> where T: Float+FloatConst {
    pub fn sinc(&self) -> HeapSlice<T> {
        let mut fb: HeapSlice<T> = HeapSlice::<T>::new();
        fb.allocate(self.length);
        for (xout,xin) in &mut fb.iter_mut().zip(self.iter()) {
            if *xin != T::zero() {
                *xout = (*xin*T::pi()).sin()/(*xin*T::pi());
            } else {
                *xout = T::one()
            }
        }
        fb
    }
}

pub fn linspace_heapslice<'a, T: 'a>(start: T, stop: T, len: usize) -> HeapSlice<T>
    where T: Float {

    let (one, zero, dx) = get_values_as_type_t::<T>(start, stop, len);
    let mut fb: HeapSlice<T> = HeapSlice::<T>::new();
    fb.allocate(len);

    unsafe {
        let mut c = zero;

        for ii in 0..len {
            let x = fb.ptr.offset((ii as isize));
            *x = start + c*dx;
            c = c + one;
        }

        return fb
    }
}


pub fn linspace_boxed_slice<'a, T: 'a>(start: T, stop: T, len: usize) -> Box<&'a mut [T]>
    where T: Float {

    let (one, zero, dx) = get_values_as_type_t::<T>(start, stop, len);

    let size = len * mem::size_of::<T>();

    unsafe {
        let ptr = heap::allocate(size, align_of::<T>()) as *mut T;

        let mut c = zero;

        for ii in 0..len {
            let x = ptr.offset((ii as isize));
            *x = start + c*dx;
            c = c + one;
        }

        let sl = slice::from_raw_parts_mut(ptr, len);
        return Box::new(sl);
    }
}

pub fn kaiser<T>(length: usize, alpha: T) -> HeapSlice<T> where T: Float+FloatConst {
    let length_t: T = num::cast(length).unwrap();
    let one = T::one();
    let two: T = num::cast(2).unwrap();
    let mut n = linspace_heapslice(T::zero(), (length_t-one), length);
    for ni in n.iter_mut() {
        let mut tmp= two*(*ni)/(length_t-one)-one;
        tmp = T::pi()*alpha* ( one- tmp.powf(two) ).sqrt();
        let tmpf64: f64 = num::cast(tmp).unwrap();
        let grr: f64 = num::cast(T::pi()*alpha).unwrap();
        let tmp2 = rgsl::bessel::I0(tmpf64) / rgsl::bessel::I0( grr );
        *ni=num::cast(tmp2).unwrap();
    }
    return n
}
