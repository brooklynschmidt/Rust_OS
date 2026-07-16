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
    unimplemented!();
}

/* 
Old _start function 

#[unsafe(no_mangle)]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");

    os::init();

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
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    os::hlt_loop();
}

*/

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



