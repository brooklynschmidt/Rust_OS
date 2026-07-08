/*
no_std -> Can't use standard library as that code depends on the OS
panic_handler -> Function that the compiler invokes on a panic
Unwinding disabled as it uses OS-specific libraries
No access to Rust runtime or crt0, must overwrite it
    - Runtime is what sets up the execution environment prior to main
*/

#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    panic!("Testing panic");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

