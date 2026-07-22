use core::{ptr, mem};
use alloc::alloc::{GlobalAlloc, Layout};
use super::{align_up, Locked};

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode{ size, next: None}
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    // Unsafe because the caller must guarantee that the heap bounds are valid and that the heap is unused.
    // Must only be called once.
    // Requires writing a node to heap memory, can only happen at runtime
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        unsafe {
            self.add_free_region(heap_start, heap_size);
        }
    }


    // Adds the given memory region to the front of the list
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // ensure the freed region is capable of holding a ListNode
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        // create a new list node and append it to the start of the linked list
        let mut node = ListNode::new(size);
        // Takes the current head.next, replaces it with None
        // Sets current head.next to this new node.next
        // node.next points to the old head.next!
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        unsafe {
            node_ptr.write(node);
            // Set head.next to be the one we just added (front of list append)
            self.head.next = Some(&mut *node_ptr)
        }
    }

    // Try to use the given region for an allocation with the given size and alignment
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            // Too small of a region
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        // Allocation must fit perfectly or fit a ListNode
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            // Rest of region too small to hold a ListNode
            return Err(());
        }

        Ok(alloc_start)
    }

    // Looks for a free region with the given size and alignment and removes it from the list
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                // Iterate through linked list
                current = current.next.as_mut().unwrap();
            }
        }
        // No suitable memory region in the linked list
        None
    }

    // Adjust the given layout so that the resulting allocated memory region is also capable of storing a ListNode
    // Returns the adjusted size and alignment as a tuple
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout.align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}


unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        // Find a region that fits
        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                unsafe {
                    // If we have excess size, insert a node to represent this excess space
                    allocator.add_free_region(alloc_end, excess_size);
                }
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }
            
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);

        unsafe { self.lock().add_free_region(ptr as usize, size) }
    }
}



