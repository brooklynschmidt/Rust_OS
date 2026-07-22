use core::mem;
use super::align_up;

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
}



