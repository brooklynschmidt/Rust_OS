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

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    os::init();

    let ptr = 0xdeadbeaf as *mut u8;
    unsafe { *ptr = 42; }

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



