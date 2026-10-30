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
    use core::ptr::NonNull;
    use uefi::prelude::*;

    const MAX_KERNEL_SIZE: u64 = 64 * 1024 * 1024;
    const KERNEL_STACK_PAGES: usize = 16;
    const MAX_LOAD_SEGMENTS: usize = 16;

    #[derive(Clone, Copy)]
    struct Allocation {
        address: NonNull<u8>,
        pages: usize,
    }

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

        let path = uefi::cstr16!("\\kernel.elf");
        let file_size = match filesystem.metadata(path) {
            Ok(metadata) => metadata.file_size(),
            Err(_) => {
                uefi::println!("Could not find /kernel.elf on the boot volume.");
                return Status::NOT_FOUND;
            }
        };
        if file_size == 0 || file_size > MAX_KERNEL_SIZE {
            uefi::println!("/kernel.elf has an unsupported file size.");
            return Status::LOAD_ERROR;
        }

        let kernel_bytes = match filesystem.read(path) {
            Ok(bytes) => bytes,
            Err(_) => {
                uefi::println!("Could not read /kernel.elf.");
                return Status::LOAD_ERROR;
            }
        };
        drop(filesystem);
        if kernel_bytes.len() as u64 != file_size || kernel_bytes.len() as u64 > MAX_KERNEL_SIZE {
            uefi::println!("/kernel.elf changed while it was being read.");
            return Status::LOAD_ERROR;
        }

        let elf = match bootloader::x86_64::elf::ElfImage::parse(&kernel_bytes) {
            Ok(elf) => elf,
            Err(_) => {
                uefi::println!("/kernel.elf is not a supported x86_64 ELF executable.");
                return Status::LOAD_ERROR;
            }
        };
        let mut allocations = [None; MAX_LOAD_SEGMENTS];
        let mut allocation_count = 0;

        for segment in elf.segments() {
            let page_size = uefi::boot::PAGE_SIZE as u64;
            let page_count = match segment
                .memory_size
                .checked_add(page_size - 1)
                .and_then(|size| usize::try_from(size / page_size).ok())
            {
                Some(count) => count,
                None => {
                    free_allocations(&allocations[..allocation_count]);
                    return Status::LOAD_ERROR;
                }
            };
            let memory_type = if segment.flags & 1 != 0 {
                uefi::boot::MemoryType::LOADER_CODE
            } else {
                uefi::boot::MemoryType::LOADER_DATA
            };
            let address = match uefi::boot::allocate_pages(
                uefi::boot::AllocateType::Address(segment.physical_address),
                memory_type,
                page_count,
            ) {
                Ok(address) => address,
                Err(_) => {
                    uefi::println!("Could not allocate a kernel segment.");
                    free_allocations(&allocations[..allocation_count]);
                    return Status::OUT_OF_RESOURCES;
                }
            };
            allocations[allocation_count] = Some(Allocation {
                address,
                pages: page_count,
            });
            allocation_count += 1;

            let destination = address.as_ptr();
            let file_start = segment.file_offset as usize;
            let file_size = segment.file_size as usize;
            let memory_size = segment.memory_size as usize;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    kernel_bytes.as_ptr().add(file_start),
                    destination,
                    file_size,
                );
                core::ptr::write_bytes(destination.add(file_size), 0, memory_size - file_size);
            }
        }

        let stack = match uefi::boot::allocate_pages(
            uefi::boot::AllocateType::AnyPages,
            uefi::boot::MemoryType::LOADER_DATA,
            KERNEL_STACK_PAGES,
        ) {
            Ok(stack) => stack,
            Err(_) => {
                uefi::println!("Could not allocate the kernel stack.");
                free_allocations(&allocations[..allocation_count]);
                return Status::OUT_OF_RESOURCES;
            }
        };
        let Some(stack_top) = (stack.as_ptr() as u64)
            .checked_add((KERNEL_STACK_PAGES as u64) * (uefi::boot::PAGE_SIZE as u64))
        else {
            let _ = unsafe { uefi::boot::free_pages(stack, KERNEL_STACK_PAGES) };
            free_allocations(&allocations[..allocation_count]);
            return Status::LOAD_ERROR;
        };

        uefi::println!("Loaded kernel ELF. Exiting UEFI boot services.");
        let entry = elf.entry();
        drop(kernel_bytes);
        let memory_map =
            unsafe { uefi::boot::exit_boot_services(uefi::boot::MemoryType::LOADER_DATA) };
        core::mem::forget(memory_map);
        unsafe { jump_to_kernel(entry, stack_top) }
    }

    fn free_allocations(allocations: &[Option<Allocation>]) {
        for allocation in allocations.iter().flatten() {
            let _ = unsafe { uefi::boot::free_pages(allocation.address, allocation.pages) };
        }
    }

    unsafe fn jump_to_kernel(entry: u64, stack_top: u64) -> ! {
        unsafe {
            core::arch::asm!(
                "cli",
                "mov rsp, {stack}",
                "and rsp, -16",
                "push 0",
                "jmp {entry}",
                stack = in(reg) stack_top,
                entry = in(reg) entry,
                options(noreturn)
            );
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
