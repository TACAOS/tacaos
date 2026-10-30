#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

pub mod memory;
pub mod sync;

#[cfg(all(target_os = "none", target_arch = "x86_64"))]
mod arch;

#[cfg(all(target_os = "none", target_arch = "x86_64"))]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut serial = arch::x86_64::serial::SerialPort::new();
    serial.initialize();
    let _ = core::fmt::Write::write_str(&mut serial, "TACAOS x86_64 kernel started\n");
    arch::x86_64::halt()
}

#[cfg(all(target_os = "none", not(target_arch = "x86_64")))]
compile_error!("the freestanding kernel currently supports x86_64 only");

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    let mut serial = arch::x86_64::serial::SerialPort::new();
    serial.initialize();
    let _ = core::fmt::Write::write_fmt(&mut serial, format_args!("Kernel panic: {info}\n"));
    arch::x86_64::halt()
}

#[cfg(not(target_os = "none"))]
fn main() {
    println!("TACAOS kernel must be built for x86_64-unknown-none.");
}
