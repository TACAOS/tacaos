# Kernel

The x86_64 kernel is a freestanding ELF with an `_start` entry point.
On startup it initializes COM1 and writes a message to the serial console.
It then disables interrupts and halts. Panics are also reported over COM1 before halting.

The architecture-independent `memory` module provides sequential and contiguous physical-frame allocation.
It accepts sorted, non-overlapping memory regions and a page size, so it can be reused with different architecture memory maps and page granules.
The bootloader supplies the memory map through `BootInfo`, but frames are not yet freed.

The architecture-independent `sync` module provides a non-poisoning spin lock and a thread-safe `OnceCell`, both built on Rust atomics.
It is not safe to acquire from an interrupt handler that may interrupt code holding the same lock.

Build it from this directory with:

```sh
rustup target add x86_64-unknown-none
cargo build --target x86_64-unknown-none
```

The resulting ELF is `../target/x86_64-unknown-none/debug/kernel`.
Copy this file to `/kernel.elf` at the root of the UEFI boot volume.
The x86_64 UEFI bootloader loads the ELF segments and transfers control to `_start`.
The shared `boot-protocol` crate defines the versioned `BootInfo` structure and normalized memory region types.
The kernel validates the passed memory map and initializes its global physical frame allocator from it.
