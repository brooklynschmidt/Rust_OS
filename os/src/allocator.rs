/* GlobalAlloc Trait Notes
Defines the functions a heap allocator must provide.
Compiler automatically inserts the appropriate calls to the trait methods when using the allocation & collection types of alloc

The alloc method takes a Layout instance as an argument
    - Layout describes the desired size & alignment the allocated memory should have
Returns a raw pointer to the first byte of the allocated memory block
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
use bump::BumpAllocator;
use linked_list_allocator::LockedHeap;
use x86_64::{structures::paging::{mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,}, VirtAddr,};

pub mod bump;
pub mod linked_list;

/*
[global_allocator] tells us which allocator to use.
It's called LockedHeap because it uses a spinlock for synchronization
Multiple threads could access the ALLOCATOR static at the same time
So, we shouldn't perform any allocations in interrupt handlers; they might happen at the same time as an in-progress allocation
*/
#[global_allocator]
//static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Bump Allocator
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());


/* 
We must create a heap memory region that the allocator can allocate memory from
We must define a virtual memory range for the heap region, then map it to a physical frame
*/

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    // The empty constructor above creates an allocator without any backing memory.
    // We must initialize this allocator after creating the heap
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

// Wrapper around spin::Mutex to permit trait implementations
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

// Aligns the given address 'addr' upwards to alignment 'align'
fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr
    } else {
        addr - remainder + align
    }
}


