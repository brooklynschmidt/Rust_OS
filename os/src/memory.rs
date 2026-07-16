use x86_64::{structures::paging::PageTable, VirtAddr,};


/*
Returns a mutable reference to the active level 4 table.

Unsafe as the caller must guarantee that the complete physical memory 
is mapped to virtual memory at 'physical_memory_offset'. 
This function must only be called once to avoid aliasing &mut references, which is UB
*/
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {

    /*
    Cr3::read
    Returns a tuple of (PhysFrame, Cr3Flags)
    We use this to get the physical memory address of the active level 4 page table

    How can we access physical memory?
        We can't while paging is active
        Otherwise, programs could circumvent memory protection and access the memory of other programs
    The only way to access the table is through some virtual page that is mapped to the physical frame at address 0x1000
    This problem of creating mappings for page table frames ia general problem, since the kernel needs to access the page tables regularly, for example, when allocating a stack for a new thread.
    */
    use x86_64::registers::control::Cr3;
    // Read the physical frame of the active level 4 table from CR3
    let (level_4_table_frame, _) = Cr3::read();

    // Physical start address
    let phys = level_4_table_frame.start_address();
    // phys + offset = virtual address where the page table frame is mapped
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // Will need to mutate the page tables 
    unsafe { &mut *page_table_ptr }
}