#![no_std]
#![no_main]

use core::panic::PanicInfo;
use swag_kernel::{serial_println, serial_print, serial::{Green, Red}, hlt_loop};
use swag_kernel::qemu::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("{}", Red("[test did not panic]"));
    exit_qemu(QemuExitCode::Failed);

    hlt_loop();
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    serial_println!("{}", Green("[ok]"));
    exit_qemu(QemuExitCode::Success);

    hlt_loop();
}

fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}
