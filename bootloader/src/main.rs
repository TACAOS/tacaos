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
        uefi::println!("TACAOS bootloader: {} UEFI", super::ARCHITECTURE);

        #[cfg(target_arch = "x86_64")]
        {
            return load_x86_64();
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            uefi::println!("Machine initialization and kernel loading are not implemented yet.");
            Status::UNSUPPORTED
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn load_x86_64() -> Status {
        let image = uefi::boot::image_handle();
        let Ok(protocol) = uefi::boot::get_image_file_system(image) else {
            uefi::println!("Could not access the boot volume.");
            return Status::NOT_FOUND;
        };
        let mut filesystem = uefi::fs::FileSystem::new(protocol);

        match filesystem.try_exists(uefi::cstr16!("/kernel.elf")) {
            Ok(true) => {
                uefi::println!(
                    "Found /kernel.elf. ELF loading and kernel handoff are not implemented."
                );
                Status::UNSUPPORTED
            }
            Ok(false) => {
                uefi::println!("/kernel.elf was not found on the boot volume.");
                Status::NOT_FOUND
            }
            Err(_) => {
                uefi::println!("Could not inspect /kernel.elf on the boot volume.");
                Status::LOAD_ERROR
            }
        }
    }
}

// only a placeholder
#[cfg(not(target_os = "uefi"))]
fn main() {
    println!(
        "TACAOS bootloader stub for {} (UEFI); no boot operations implemented.",
        ARCHITECTURE
    );
}
