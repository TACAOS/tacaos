#![cfg_attr(target_os = "uefi", no_std)]
#![cfg_attr(target_os = "uefi", no_main)]

#[cfg(target_arch = "x86")]
const ARCHITECTURE: &str = "x86";
#[cfg(target_arch = "x86_64")]
const ARCHITECTURE: &str = "x86_64";
#[cfg(target_arch = "arm")]
const ARCHITECTURE: &str = "AArch32";
#[cfg(target_arch = "aarch64")]
const ARCHITECTURE: &str = "AArch64";
#[cfg(not(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "arm",
    target_arch = "aarch64"
)))]
const ARCHITECTURE: &str = "unsupported";

#[cfg(target_os = "uefi")]
mod firmware {
    use uefi::prelude::*;

    #[entry]
    fn main() -> Status {
        uefi::println!("TACAOS bootloader stub: {} UEFI", super::ARCHITECTURE);
        uefi::println!("Machine initialization and kernel loading are not implemented yet.");
        Status::UNSUPPORTED
    }
}

// only a placeholder
#[cfg(not(target_os = "uefi"))]
fn main() {
    println!("TACAOS bootloader stub for {} (UEFI); no boot operations implemented.", ARCHITECTURE);
}
