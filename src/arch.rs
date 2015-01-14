use core::prelude::*;
use core::mem::{size_of, align_of};
use core::cmp::max;
use core::ptr;

use stack::Stack;

#[allow(non_camel_case_types)]
pub type uintptr_t = u64;

pub struct Registers {
  rsp: *mut uintptr_t
}

impl Copy for Registers {}

#[inline(always)]
pub unsafe fn swap(regs: &mut Registers) {
  asm!(include_str!("swap.s")
        :
        : "{rdi}" (&mut regs.rsp)
        : "rax", "rbx", "rcx", "rdx", "rsi", "rdi",
          "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
          "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
          "xmm8", "xmm9", "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15",
          "cc"
        : "volatile");
}

#[inline]
pub unsafe fn initialize_call_frame<S, F>(stack: &mut S, f: F) -> Registers where S: Stack, F: FnOnce() {
  let sp_limit = stack.limit();
  let mut sp = stack.top() as *mut uintptr_t;
  let f_ptr = push(&mut sp, f);

  asm!(include_str!("init.s")
        : "={rdi}"(sp)
        : "{rdi}" (sp),
          "{rsi}" (rust_trampoline::<F>),
          "{rdx}" (f_ptr),
          "{rcx}" (sp_limit)
        :
        : "volatile");

  Registers { rsp: sp }
}

unsafe extern "C" fn rust_trampoline<F: FnOnce()>(f: *const F) {
  ptr::read(f)()
}

unsafe fn push<T>(spp: &mut *mut uintptr_t, value: T) -> *mut T {
  let mut sp = *spp as *mut T;
  sp = offset_mut(sp, -1);
  sp = align_down_mut(sp, max(align_of::<T>(), 16));
  *sp = value;
  *spp = sp as *mut uintptr_t;
  sp
}

fn align_down_mut<T>(sp: *mut T, n: usize) -> *mut T {
  let sp = (sp as usize) & !(n - 1);
  sp as *mut T
}

// ptr::offset_mut is positive ints only
pub fn offset_mut<T>(ptr: *mut T, count: isize) -> *mut T {
  (ptr as isize + count * (size_of::<T>() as isize)) as *mut T
}
