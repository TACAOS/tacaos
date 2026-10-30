pub mod serial;

pub fn halt() -> ! {
    loop {
        unsafe { core::arch::asm!("cli", "hlt", options(nomem, nostack)) };
    }
}
