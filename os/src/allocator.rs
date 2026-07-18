/* GlobalAlloc Trait Notes
Defines the functions a heap allocator must provide.
Compiler automatically inserts the appropriate calls to the trait methods when using the allocation & collection types of alloc

The alloc method takes a Layout instance as an argument
    - Layout describes the desired size & alignment the allocated memory should have
Returns a raw pointer to the first byte of the allocateed memory block
Returns null pointer to signal an error

The dealloc method is responsible for freeing a memory block
Receives the pointer returned by alloc and the Layout used for allocation

The following two methods have default implementations: 
The alloc_zeroed method is equivalent to calling alloc and setting the allocated memory block to zero.

The realloc method allows growing and shrinking an allocation
It allocates a new memory block with the desired size and performs a copy

Both the trait itself and all the methods are unsafe
    The programmer must guarantee that the trait implemention for an allocator type is correct.
    The alloc method can't return a memory block that is already used somewhere
The caller must ensure various invariants when calling the methods
    The Layout passed to alloc must be a non-zero size.
*/

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

#[global_allocator]
static ALLOCATOR: Dummy = Dummy;

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("Dealloc doesn't work for a dummy allocator");
    }
}


