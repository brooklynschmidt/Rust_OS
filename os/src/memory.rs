use x86_64::{structures::paging::PageTable, structures::paging::OffsetPageTable, VirtAddr, PhysAddr,};

/*
Mapper trait -> Generic over the page size & provides functions that operate on pages
    translate_page -> translates a given page to a frame of the same size
    map_to -> creates a new mapping in the page table
Translate trait -> Provides functions that work with multiple page sizes.

Three types implement these traits
1. OffsetPageTable -> Assumes that the complete physical memory is mapped to the virtual address space at some offset
2. MappedPageTable -> Only requires that each page table frame is mapped to the virtual address spaec at a calculable address
3. RecursivePageTable -> Allows us to access page table frames through recursive page tables
*/
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

/*
Returns a mutable reference to the active level 4 table.

Unsafe as the caller must guarantee that the complete physical memory 
is mapped to virtual memory at 'physical_memory_offset'. 
This function must only be called once to avoid aliasing &mut references, which is UB
*/
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {

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

/*
DEPRECATED: Using OffsetPageTable 

Translates the given virtual address to the mapped physical address
Returns None if the address is not mapped

This function is unsafe as the caller must guarantee that the complete physical memory
is mapped to virtual memory at the passed 'physical_memory_offset'

pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr>
{
    // Limit scope of unsafe
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr>
{
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    // traverse the multi-level page table
    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        // Read the page table entry and update 'frame'
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }
    // Calculate the physical address by adding the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

*/
