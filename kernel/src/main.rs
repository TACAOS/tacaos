#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

pub mod memory;
pub mod sync;

#[cfg(all(target_os = "none", target_arch = "x86_64"))]
use boot_protocol::{BOOT_INFO_FLAG_MEMORY_MAP_TRUNCATED, BootInfo};

#[cfg(all(target_os = "none", target_arch = "x86_64"))]
mod arch;

#[cfg(all(target_os = "none", target_arch = "x86_64"))]
#[unsafe(no_mangle)]
pub extern "C" fn _start(boot_info_pointer: *const BootInfo) -> ! {
    let mut serial = arch::x86_64::serial::SerialPort::new();
    serial.initialize();
    let _ = core::fmt::Write::write_str(&mut serial, "TACAOS x86_64 kernel started\n");

    let Some(boot_info) = (unsafe { boot_info_pointer.as_ref() }) else {
        let _ = core::fmt::Write::write_str(&mut serial, "Missing boot information\n");
        arch::x86_64::halt();
    };
    if !boot_info.is_valid() {
        let _ = core::fmt::Write::write_str(&mut serial, "Invalid boot information\n");
        arch::x86_64::halt();
    }
    if boot_info.flags & BOOT_INFO_FLAG_MEMORY_MAP_TRUNCATED != 0 {
        let _ = core::fmt::Write::write_str(&mut serial, "Boot memory map was truncated\n");
        arch::x86_64::halt();
    }

    let regions: &'static [memory::MemoryRegion] = unsafe { boot_info.regions() };
    match memory::initialize(regions, boot_info.page_size) {
        Ok(()) => {
            let _ = core::fmt::Write::write_fmt(
                &mut serial,
                format_args!(
                    "Initialized frame allocator with {} regions\n",
                    regions.len()
                ),
            );
        }
        Err(_) => {
            let _ = core::fmt::Write::write_str(&mut serial, "Invalid physical memory map\n");
        }
    }
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
