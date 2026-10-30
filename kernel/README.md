# Kernel

The x86_64 kernel is a freestanding ELF with an `_start` entry point.
On startup it initializes COM1 and writes a message to the serial console.
It then disables interrupts and halts. Panics are also reported over COM1 before halting.

The architecture-independent `memory` module provides sequential and contiguous physical-frame allocation.
It accepts sorted, non-overlapping memory regions and a page size, so it can be reused with different architecture memory maps and page granules.
It does not yet free frames or consume a bootloader memory map.

The architecture-independent `sync` module provides a non-poisoning spin lock and a thread-safe `OnceCell`, both built on Rust atomics.
It is not safe to acquire from an interrupt handler that may interrupt code holding the same lock.

Build it from this directory with:

```sh
rustup target add x86_64-unknown-none
cargo build --target x86_64-unknown-none
```

The resulting ELF is `../target/x86_64-unknown-none/debug/kernel`.
The initial bootloader image convention expects this file to be copied to `/kernel.elf` on the boot volume. 
The bootloader does not yet load ELF segments or transfer control to the kernel.
