// Fizzy: A fast WebAssembly interpreter
// Copyright 2019-2020 The Fizzy Authors.
// SPDX-License-Identifier: Apache-2.0

//! This is a Rust interface to [Fizzy](https://github.com/wasmx/fizzy), a WebAssembly virtual machine.

mod sys;

use std::ffi::CString;
use std::ptr::NonNull;

/// Parse and validate the input according to WebAssembly 1.0 rules. Returns true if the supplied input is valid.
pub fn validate<T: AsRef<[u8]>>(input: T) -> bool {
    unsafe { sys::fizzy_validate(input.as_ref().as_ptr(), input.as_ref().len()) }
}

/// A parsed and validated WebAssembly 1.0 module.
// NOTE: cannot use NonNull here given this is *const
pub struct Module(*const sys::FizzyModule);

impl Drop for Module {
    fn drop(&mut self) {
        unsafe { sys::fizzy_free_module(self.0) }
    }
}

/// Parse and validate the input according to WebAssembly 1.0 rules.
pub fn parse<T: AsRef<[u8]>>(input: &T) -> Result<Module, ()> {
    let ptr = unsafe { sys::fizzy_parse(input.as_ref().as_ptr(), input.as_ref().len()) };
    if ptr.is_null() {
        return Err(());
    }
    Ok(Module { 0: ptr })
}

/// An instance of a module.
pub struct Instance(NonNull<sys::FizzyInstance>);

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { sys::fizzy_free_instance(self.0.as_ptr()) }
    }
}

impl Module {
    /// Create an instance of a module.
    // TODO: support imported functions
    pub fn instantiate(self) -> Result<Instance, ()> {
        if self.0.is_null() {
            return Err(());
        }
        let ptr = unsafe {
            sys::fizzy_instantiate(
                self.0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                0,
            )
        };
        // Forget Module (and avoid calling drop) because it has been consumed by instantiate (even if it failed).
        core::mem::forget(self);
        if ptr.is_null() {
            return Err(());
        }
        Ok(Instance {
            0: unsafe { NonNull::new_unchecked(ptr) },
        })
    }
}

// TODO: add a proper wrapper (enum type for Value, and Result<Value, Trap> for ExecutionResult)

/// A WebAssembly value of i32/i64/f32/f64.
pub type Value = sys::FizzyValue;

// NOTE: the union does not have i32
impl Value {
    pub fn as_i32(&self) -> i32 {
        unsafe { self.i64 as i32 }
    }
    pub fn as_u32(&self) -> u32 {
        unsafe { self.i64 as u32 }
    }
    pub fn as_i64(&self) -> i64 {
        unsafe { self.i64 as i64 }
    }
    pub fn as_u64(&self) -> u64 {
        unsafe { self.i64 }
    }
    pub fn as_f32(&self) -> f32 {
        unsafe { self.f32 }
    }
    pub fn as_f64(&self) -> f64 {
        unsafe { self.f64 }
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value { i64: v as u64 }
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value { i64: v as u64 }
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value { i64: v as u64 }
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value { i64: v }
    }
}
impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value { f32: v }
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value { f64: v }
    }
}

/// The result of an execution.
pub struct ExecutionResult(sys::FizzyExecutionResult);

impl ExecutionResult {
    /// True if execution has resulted in a trap.
    pub fn trapped(&self) -> bool {
        self.0.trapped
    }

    /// The optional return value. Only a single return value is allowed in WebAssembly 1.0.
    pub fn value(&self) -> Option<Value> {
        if self.0.has_value {
            assert!(!self.0.trapped);
            Some(self.0.value)
        } else {
            None
        }
    }
}

impl From<ExecutionResult> for sys::FizzyExecutionResult {
    fn from(v: ExecutionResult) -> Self {
        v.0
    }
}

impl Instance {
    fn checked_memory_range(&self, offset: u32) -> core::ops::Range<usize> {
        let offset = offset as usize;
        let size = 0 as usize;
        offset..offset + size
    }

    pub fn memory_size(&self) -> usize {
        unsafe { sys::fizzy_get_instance_memory_size(self.0.as_ptr()) }
    }

    pub fn memory_get_into(&mut self, offset: u32, target: &mut [u8]) -> Result<(), ()> {
        let mem = unsafe {
            std::slice::from_raw_parts(
                sys::fizzy_get_instance_memory_data(self.0.as_ptr()),
                sys::fizzy_get_instance_memory_size(self.0.as_ptr()),
            )
        };
        target.copy_from_slice(&mem[self.checked_memory_range(offset)]);
        Ok(())
    }

    pub fn memory_set(&mut self, offset: u32, value: &[u8]) -> Result<(), ()> {
        let mem = unsafe {
            std::slice::from_raw_parts_mut(
                sys::fizzy_get_instance_memory_data(self.0.as_ptr()),
                sys::fizzy_get_instance_memory_size(self.0.as_ptr()),
            )
        };
        mem[self.checked_memory_range(offset)].copy_from_slice(value);
        Ok(())
    }

    /// Find index of exported function by name.
    pub fn find_exported_function_index(&self, name: &str) -> Option<u32> {
        let module = unsafe { sys::fizzy_get_instance_module(self.0.as_ptr()) };
        let name = CString::new(name).expect("CString::new failed");
        let mut func_idx: u32 = 0;
        let found = unsafe {
            sys::fizzy_find_exported_function_index(module, name.as_ptr(), &mut func_idx)
        };
        if found {
            Some(func_idx)
        } else {
            None
        }
    }

    /// Unsafe execution of a given function index `func_idx` with the given values `args`.
    ///
    /// An invalid index, invalid inputs, or invalid depth can cause undefined behaviour.
    ///
    /// The `depth` argument sets the initial call depth. Should be set to `0`.
    ///
    /// # Safety
    /// This function expects a valid `func_idx`, appropriate number of `args`, and valid `depth`.
    pub unsafe fn unsafe_execute(
        &mut self,
        func_idx: u32,
        args: &[Value],
        depth: i32,
    ) -> ExecutionResult {
        ExecutionResult {
            0: sys::fizzy_execute(self.0.as_ptr(), func_idx, args.as_ptr(), depth),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_wasm() {
        // Empty
        assert_eq!(validate(&[]), false);
        // Too short
        assert_eq!(validate(&[0x00]), false);
        // Valid
        assert_eq!(
            validate(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]),
            true
        );
        // Invalid version
        assert_eq!(
            validate(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x01]),
            false
        );
    }

    #[test]
    fn parse_wasm() {
        assert!(parse(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]).is_ok());
        assert!(parse(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x01]).is_err());
    }

    #[test]
    fn instantiate_wasm() {
        let module = parse(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
        assert!(module.is_ok());
        let instance = module.unwrap().instantiate();
        assert!(instance.is_ok());
    }

    #[test]
    fn find_exported_function_index() {
        /* wat2wasm
        (module
          (func $f (export "foo") (result i32) (i32.const 42))
          (global (export "g1") i32 (i32.const 0))
          (table (export "tab") 0 anyfunc)
          (memory (export "mem") 1 2)
        )
        */
        let input = hex::decode(
        "0061736d010000000105016000017f030201000404017000000504010101020606017f0041000b07180403666f6f00000267310300037461620100036d656d02000a06010400412a0b").unwrap();

        let module = parse(&input);
        assert!(module.is_ok());
        let instance = module.unwrap().instantiate();
        assert!(instance.is_ok());
        let mut instance = instance.unwrap();

        let func_idx = instance.find_exported_function_index(&"foo");
        assert!(func_idx.is_some());
        assert_eq!(func_idx.unwrap(), 0);

        assert!(instance.find_exported_function_index(&"bar").is_none());
        assert!(instance.find_exported_function_index(&"g1").is_none());
        assert!(instance.find_exported_function_index(&"tab").is_none());
        assert!(instance.find_exported_function_index(&"mem").is_none());
    }

    #[test]
    fn unsafe_execute_wasm() {
        /* wat2wasm
          (func)
          (func (result i32) i32.const 42)
          (func (param i32 i32) (result i32)
            (i32.div_u (local.get 0) (local.get 1))
          )
          (func unreachable)
        */
        let input = hex::decode("0061736d01000000010e036000006000017f60027f7f017f030504000102000a150402000b0400412a0b0700200020016e0b0300000b").unwrap();
        let module = parse(&input);
        assert!(module.is_ok());
        let instance = module.unwrap().instantiate();
        assert!(instance.is_ok());
        let mut instance = instance.unwrap();

        let result = unsafe { instance.unsafe_execute(0, &[], 0) };
        assert!(!result.trapped());
        assert!(!result.value().is_some());

        let result = unsafe { instance.unsafe_execute(1, &[], 0) };
        assert!(!result.trapped());
        assert!(result.value().is_some());
        assert_eq!(result.value().unwrap().as_i32(), 42);

        // Explicit type specification
        let result =
            unsafe { instance.unsafe_execute(2, &[(42 as i32).into(), (2 as i32).into()], 0) };
        assert!(!result.trapped());
        assert!(result.value().is_some());
        assert_eq!(result.value().unwrap().as_i32(), 21);

        // Implicit i64 types (even though the code expects i32)
        let result = unsafe { instance.unsafe_execute(2, &[42.into(), 2.into()], 0) };
        assert!(!result.trapped());
        assert!(result.value().is_some());
        assert_eq!(result.value().unwrap().as_i32(), 21);

        let result = unsafe { instance.unsafe_execute(3, &[], 0) };
        assert!(result.trapped());
        assert!(!result.value().is_some());
    }
}
