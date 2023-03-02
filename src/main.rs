#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    eprintln!("{}", _info);
    loop {}
}


mod vga_buffer;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World!!!!!!!!! FINALFLKWJFKLJWROIFJ\n{}", core::f64::consts::PI);

    eprintln!("wtf?");

    println!("E = {} !!!!", core::f64::consts::E);

    loop {}
}

