// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//! Set and unset common attributes on LLVM values.

use libc::c_uint;
use llvm::{self, ValueRef};
pub use syntax::attr::InlineAttr;
use syntax::ast;
use context::CrateContext;

/// Mark LLVM function to use provided inline heuristic.
#[inline]
pub fn inline(val: ValueRef, inline: InlineAttr) {
    use self::InlineAttr::*;
    match inline {
        Hint   => llvm::SetFunctionAttribute(val, llvm::Attribute::InlineHint),
        Always => llvm::SetFunctionAttribute(val, llvm::Attribute::AlwaysInline),
        Never  => llvm::SetFunctionAttribute(val, llvm::Attribute::NoInline),
        None   => {
            let attr = llvm::Attribute::InlineHint |
                       llvm::Attribute::AlwaysInline |
                       llvm::Attribute::NoInline;
            llvm::RemoveFunctionAttributes(val, attr)
        },
    };
}

/// Tell LLVM to emit or not emit the information necessary to unwind the stack for the function.
#[inline]
pub fn emit_uwtable(val: ValueRef, emit: bool) {
    if emit {
        llvm::SetFunctionAttribute(val, llvm::Attribute::UWTable);
    } else {
        llvm::RemoveFunctionAttributes(val, llvm::Attribute::UWTable);
    }
}

/// Tell LLVM whether the function can or cannot unwind.
#[inline]
pub fn unwind(val: ValueRef, can_unwind: bool) {
    if can_unwind {
        llvm::RemoveFunctionAttributes(val, llvm::Attribute::NoUnwind);
    } else {
        llvm::SetFunctionAttribute(val, llvm::Attribute::NoUnwind);
    }
}

/// Tell LLVM whether it should optimise function for size.
#[inline]
#[allow(dead_code)] // possibly useful function
pub fn set_optimize_for_size(val: ValueRef, optimize: bool) {
    if optimize {
        llvm::SetFunctionAttribute(val, llvm::Attribute::OptimizeForSize);
    } else {
        llvm::RemoveFunctionAttributes(val, llvm::Attribute::OptimizeForSize);
    }
}

/// Tell LLVM if this function should be 'naked', i.e. skip the epilogue and prologue.
#[inline]
pub fn naked(val: ValueRef, is_naked: bool) {
    if is_naked {
        llvm::SetFunctionAttribute(val, llvm::Attribute::Naked);
    } else {
        llvm::RemoveFunctionAttributes(val, llvm::Attribute::Naked);
    }
}

pub fn set_frame_pointer_elimination(ccx: &CrateContext, llfn: ValueRef) {
    // FIXME: #11906: Omitting frame pointers breaks retrieving the value of a
    // parameter.
    if ccx.sess().must_not_eliminate_frame_pointers() {
        unsafe {
            let attr = "no-frame-pointer-elim\0".as_ptr() as *const _;
            let val = "true\0".as_ptr() as *const _;
            llvm::LLVMRustAddFunctionAttrStringValue(llfn,
                                                     llvm::FunctionIndex as c_uint,
                                                     attr,
                                                     val);
        }
    }
}

/// Composite function which sets LLVM attributes for function depending on its AST (#[attribute])
/// attributes.
pub fn from_fn_attrs(ccx: &CrateContext, attrs: &[ast::Attribute], llfn: ValueRef) {
    use syntax::attr::*;
    inline(llfn, find_inline_attr(Some(ccx.sess().diagnostic()), attrs));

    set_frame_pointer_elimination(ccx, llfn);

    for attr in attrs {
        if attr.check_name("cold") {
            llvm::Attributes::default().set(llvm::Attribute::Cold)
                .apply_llfn(llvm::FunctionIndex as usize, llfn)
        } else if attr.check_name("naked") {
            naked(llfn, true);
        } else if attr.check_name("allocator") {
            llvm::Attributes::default().set(llvm::Attribute::NoAlias)
                .apply_llfn(llvm::ReturnIndex as usize, llfn)
        } else if attr.check_name("unwind") {
            unwind(llfn, true);
        }
    }
}
