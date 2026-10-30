# Bootloader

The TACAOS bootloader is responsible for initializing the machine and loading the kernel.
The project plans to support these architecture targets:

- x86
- x86_64
- AArch32
- AArch64

UEFI is the initial firmware target. Legacy BIOS support is planned for a later stage.
The current Rust crate is only a scaffold: it identifies the build architecture and has a UEFI entry point, but does not yet initialize hardware or load a kernel. 
Architecture-specific initialization and handoff will be added as those stages are implemented.

For example, to build the current UEFI application for x86_64:

```sh
rustup target add x86_64-unknown-uefi
cargo build -p bootloader --target x86_64-unknown-uefi
```

