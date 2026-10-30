# Bootloader

The TACAOS bootloader is responsible for initializing the machine and loading the kernel.
The project plans to support these architecture targets:

- x86
- x86_64
- AArch32
- AArch64

UEFI is the initial firmware target. Legacy BIOS support is planned for a later stage.
On x86_64 UEFI, the bootloader reads `/kernel.elf`, validates its ELF64 load segments, allocates the segments at their linked physical addresses, and creates a 64 KiB stack.
It then exits UEFI boot services and transfers control to the ELF entry point.
The current handoff passes no boot information, and the kernel relies on the inherited UEFI identity mappings.
Other architectures currently report that their boot flow is not implemented.

The x86_64 loader accepts static little-endian ELF64 executables for the AMD64 machine type, with at most 16 load segments and a maximum image size of 64 MiB.
The kernel is linked at physical and virtual address `0x100000`, and the bootloader requires page-aligned, non-overlapping load segments.
Copy the kernel ELF to `/kernel.elf` at the root of the boot volume.

Build the kernel from the `kernel/` directory with `cargo build --target x86_64-unknown-none`.
The resulting file is `target/x86_64-unknown-none/debug/kernel` relative to the repository root.

For example, to build the current UEFI application for x86_64:

```sh
rustup target add x86_64-unknown-uefi
cargo build -p bootloader --target x86_64-unknown-uefi
```
