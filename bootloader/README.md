# Bootloader

The TACAOS bootloader is responsible for initializing the machine and loading the kernel.
The project plans to support these architecture targets:

- x86
- x86_64
- AArch32
- AArch64

UEFI is the initial firmware target. Legacy BIOS support is planned for a later stage.
The current Rust crate is an early scaffold. On x86_64 UEFI, it checks the boot volume for `/kernel.elf`.
It does not yet initialize hardware, load the ELF image, or hand off control to the kernel.
Other architectures currently report that their boot flow is not implemented.

The initial x86_64 kernel image convention is an ELF64 file named `/kernel.elf` at the root of the boot volume.
ELF loading and the kernel handoff ABI still need to be implemented.

For example, to build the current UEFI application for x86_64:

```sh
rustup target add x86_64-unknown-uefi
cargo build -p bootloader --target x86_64-unknown-uefi
```
