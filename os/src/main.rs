/*
no_std -> Can't use standard library as that code depends on the OS
panic_handler -> Function that the compiler invokes on a panic
Unwinding disabled as it uses OS-specific libraries
No access to Rust runtime or crt0, must overwrite it
    - Runtime is what sets up the execution environment prior to main
*/

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use core::panic::PanicInfo;
use os::println;
use bootloader::{BootInfo, entry_point};

/*
entry_point macro provides a type-checked way to define a Rust function as the entry point
*/
entry_point!(kernel_main);

/*
BootInfo has two fields: memory_map and physical_memory_offset

memory_map tells the kernel how much physical memory is availabl in the system
Also tells us which memory regions are reserved for devices such as VGA hardware

physical_memory_offset tells us the virtual start address of the physical memory mapping
*/
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;
    use x86_64::structures::paging::Page;
    use os::memory;
    use os::allocator;

    println!("Hello World{}", "!");
    os::init();
    
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    }; 

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap Initialization failed");
    
    let heap_value = Box::new(41);
    println!("Heap_value at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }

    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1,2,3]);
    let cloned_reference = reference_counted.clone();
    println!("Current ref count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("Ref count is {} now", Rc::strong_count(&cloned_reference));
    /*
    Testing translate function 
    // Using "0" since we know it is used.
    // Typically do not want to do this since this guarantees "NULL"
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // Write the string 'New!' to the screen via the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // We write a value to offset 400 as we don't write to the start of the page, because the top line of the VGA buffer is directly shifted off the screen by the next println!
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};
    

    //  VGA buffer page, code page, stack page, virtual address mapped to physical address 0
    let addresses = [
        0xb8000, 0x201008, 0x0100_0020_1a10, boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }
    */

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}



