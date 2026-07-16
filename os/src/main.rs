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
    use os::memory::active_level_4_table;
    use x86_64::VirtAddr;
    use x86_64::structures::paging::PageTable;
    use os::memory::translate_addr;

    println!("Hello World{}", "!");
    os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    //  VGA buffer page, code page, stack page, virtual address mapped to physical address 0
    let addresses = [
        0xb8000, 0x201008, 0x0100_0020_1a10, boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        println!("{:?} -> {:?}", virt, phys);
    }

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



