use core::fmt;

const COM1: u16 = 0x3f8;
const LINE_STATUS: u16 = COM1 + 5;
const TRANSMIT_READY: u8 = 1 << 5;
const WAIT_LIMIT: usize = 100_000;

pub struct SerialPort;

impl SerialPort {
    pub const fn new() -> Self {
        Self
    }

    pub fn initialize(&mut self) {
        unsafe {
            Self::outb(COM1 + 1, 0x00);
            Self::outb(COM1 + 3, 0x80);
            Self::outb(COM1, 0x03);
            Self::outb(COM1 + 1, 0x00);
            Self::outb(COM1 + 3, 0x03);
            Self::outb(COM1 + 2, 0xc7);
            Self::outb(COM1 + 4, 0x03);
        }
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), fmt::Error> {
        for _ in 0..WAIT_LIMIT {
            if unsafe { Self::inb(LINE_STATUS) } & TRANSMIT_READY != 0 {
                unsafe { Self::outb(COM1, byte) };
                return Ok(());
            }
            core::hint::spin_loop();
        }

        Err(fmt::Error)
    }

    unsafe fn outb(port: u16, value: u8) {
        unsafe {
            core::arch::asm!(
                "out dx, al",
                in("dx") port,
                in("al") value,
                options(nomem, nostack, preserves_flags)
            );
        }
    }

    unsafe fn inb(port: u16) -> u8 {
        let value: u8;
        unsafe {
            core::arch::asm!(
                "in al, dx",
                in("dx") port,
                out("al") value,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r')?;
            }
            self.write_byte(byte)?;
        }
        Ok(())
    }
}
