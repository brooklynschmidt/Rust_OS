use x86_64::{structures::paging::PageTable, structures::paging::OffsetPageTable, VirtAddr, PhysAddr,};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

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

use x86_64::structures::paging::{Page, PhysFrame, Mapper, Size4KiB, FrameAllocator};

// Creates a mapping for the given page to frame '0xb8000'
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    // Unsafe because the caller must ensure that the frame is not already in use
    // Mapping the same frame twice can result in UB
    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };

    // Must flush from the TLB, returns a MapperFlush type
    map_to_result.expect("map_to failed").flush();
}

// FrameAllocator that always returns None
pub struct EmptyFrameAllocator;

// Unsafe because the implementor must guarantee that the allocator yields only unused frames
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

// A FrameAllocator that returns usable frames from the bootloader's memory map
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize, // next frame the allocator should return
}

impl BootInfoFrameAllocator {
    /*
    Create a FrameAllocator from the passed memory map

    Unsafe because the caller must guarantee that the passed memory map is valid
    The main requirement is that all frames that are marked as "USABLE" are really unused.
    */
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    // Returns an iterator over the usable frames specified in the memory map
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // Get usable regions
        let regions = self.memory_map_iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // Map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // Transform to iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // Create PhysFrame types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    // Recreates the usable_frame allocator on every allocation
    // Might want to store this as a struct field to optimize
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
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
