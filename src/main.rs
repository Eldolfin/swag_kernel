#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(swag_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use swag_kernel::{println, hlt_loop};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Swag Kernel!");

    swag_kernel::init();

    #[cfg(test)]
    test_main();

    println!("No crash!");
    
    hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use swag_kernel::eprintln;
    eprintln!("{}", info);

    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    swag_kernel::test_panic_handler(info)
}


#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
