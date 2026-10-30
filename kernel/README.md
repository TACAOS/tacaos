# Kernel

The x86_64 kernel is a freestanding ELF with an `_start` entry point.
On startup it initializes COM1 and writes a message to the serial console.
It then disables interrupts and halts. Panics are also reported over COM1 before halting.

In QEMU, attach the serial console with `-serial stdio` to see the startup message.

Build it from this directory with:

```sh
rustup target add x86_64-unknown-none
cargo build --target x86_64-unknown-none
```

The resulting ELF is `../target/x86_64-unknown-none/debug/kernel`.
The initial bootloader image convention expects this file to be copied to `/kernel.elf` on the boot volume. 
The bootloader does not yet load ELF segments or transfer control to the kernel.
