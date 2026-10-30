#![no_std]

/// Magic value identifying the TACAOS boot information structure.
pub const BOOT_INFO_MAGIC: [u8; 8] = *b"TACAOSBI";
/// Current version of the boot information structure.
pub const BOOT_INFO_VERSION: u32 = 1;
/// The memory map was too large for the bootloader's reserved buffer.
pub const BOOT_INFO_FLAG_MEMORY_MAP_TRUNCATED: u32 = 1;

/// The allocation policy for a physical memory region.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRegionKind {
    Usable = 1,
    Reserved = 2,
}

/// A physical memory region supplied by the boot environment.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRegion {
    /// Inclusive start address of the region.
    pub start: u64,
    /// Length of the region in bytes.
    pub length: u64,
    /// Whether the region may be used for general allocations.
    pub kind: MemoryRegionKind,
}

/// Versioned data passed by the bootloader to the kernel entry point.
#[repr(C)]
pub struct BootInfo {
    pub magic: [u8; 8],
    pub version: u32,
    pub struct_size: u32,
    pub memory_regions: *const MemoryRegion,
    pub memory_region_count: usize,
    pub page_size: u64,
    pub flags: u32,
}

impl BootInfo {
    /// Creates a boot information structure for a memory region array.
    pub const fn new(
        memory_regions: *const MemoryRegion,
        memory_region_count: usize,
        page_size: u64,
        flags: u32,
    ) -> Self {
        Self {
            magic: BOOT_INFO_MAGIC,
            version: BOOT_INFO_VERSION,
            struct_size: core::mem::size_of::<Self>() as u32,
            memory_regions,
            memory_region_count,
            page_size,
            flags,
        }
    }

    /// Checks the boot information header and basic field invariants.
    pub fn is_valid(&self) -> bool {
        self.magic == BOOT_INFO_MAGIC
            && self.version == BOOT_INFO_VERSION
            && self.struct_size as usize == core::mem::size_of::<Self>()
            && self.page_size.is_power_of_two()
            && self
                .memory_region_count
                .checked_mul(core::mem::size_of::<MemoryRegion>())
                .is_some()
            && (self.memory_region_count == 0
                || !self.memory_regions.is_null()
                    && (self.memory_regions as usize)
                        .is_multiple_of(core::mem::align_of::<MemoryRegion>()))
    }

    /// Borrows the memory region array described by this structure.
    ///
    /// # Safety
    /// The pointer must be aligned and valid for `memory_region_count` elements for the returned lifetime.
    pub unsafe fn regions<'a>(&self) -> &'a [MemoryRegion] {
        let pointer = if self.memory_region_count == 0 {
            core::ptr::NonNull::dangling().as_ptr()
        } else {
            self.memory_regions
        };
        unsafe { core::slice::from_raw_parts(pointer, self.memory_region_count) }
    }
}

#[cfg(test)]
mod tests {
    use super::{BootInfo, MemoryRegion, MemoryRegionKind};

    #[test]
    fn validates_boot_info_header_and_region_reference() {
        let regions = [MemoryRegion {
            start: 0x1000,
            length: 0x2000,
            kind: MemoryRegionKind::Usable,
        }];
        let info = BootInfo::new(regions.as_ptr(), regions.len(), 4096, 0);

        assert!(info.is_valid());
        assert_eq!(unsafe { info.regions() }, regions);
    }

    #[test]
    fn rejects_invalid_page_size_and_null_region_pointer() {
        let invalid_page_size = BootInfo::new(core::ptr::NonNull::dangling().as_ptr(), 0, 3000, 0);
        assert!(!invalid_page_size.is_valid());

        let null_regions = BootInfo::new(core::ptr::null(), 1, 4096, 0);
        assert!(!null_regions.is_valid());
    }
}
