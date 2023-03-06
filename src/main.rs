#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(swag_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use swag_kernel::allocator;
use swag_kernel::println;
use swag_kernel::hlt_loop;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use swag_kernel::memory::BootInfoFrameAllocator;
    use swag_kernel::memory;
    use x86_64::VirtAddr;

    println!("Swag Kernel!");

    swag_kernel::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");


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
