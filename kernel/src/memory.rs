//! Physical frame allocation.

/// A physical memory region supplied by the boot environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRegion {
    /// Inclusive start address of the region.
    pub start: u64,
    /// Length of the region in bytes.
    pub length: u64,
    /// Whether the region may be used for general allocations.
    pub kind: MemoryRegionKind,
}

/// The allocation policy for a physical memory region.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRegionKind {
    Usable,
    Reserved,
}

/// A physical page-sized frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysFrame {
    /// Physical address of the frame's first byte.
    pub address: u64,
}

/// An invalid memory map or page size passed to [`FrameAllocator::new`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameAllocatorError {
    InvalidPageSize,
    RegionAddressOverflow,
    RegionsNotOrderedOrOverlapping,
}

/// Sequentially allocates physical frames from usable memory regions.
///
/// Regions must be sorted by start address and must not overlap.
/// The allocator aligns region starts to `page_size` and never returns reserved memory.
/// Frames are not freed; this allocator is intended as a simple early-boot allocator until a reclaiming allocator is added.
pub struct FrameAllocator<'a> {
    regions: &'a [MemoryRegion],
    page_size: u64,
    region_index: usize,
    next_address: Option<u64>,
}

impl<'a> FrameAllocator<'a> {
    /// Creates an allocator using a power-of-two page size in bytes.
    pub fn new(regions: &'a [MemoryRegion], page_size: u64) -> Result<Self, FrameAllocatorError> {
        if !page_size.is_power_of_two() {
            return Err(FrameAllocatorError::InvalidPageSize);
        }

        let mut previous_end = 0;
        for (index, region) in regions.iter().enumerate() {
            let end = region
                .start
                .checked_add(region.length)
                .ok_or(FrameAllocatorError::RegionAddressOverflow)?;
            if index > 0 && region.start < previous_end {
                return Err(FrameAllocatorError::RegionsNotOrderedOrOverlapping);
            }
            previous_end = end;
        }

        Ok(Self {
            regions,
            page_size,
            region_index: 0,
            next_address: None,
        })
    }

    /// Returns the next usable physical frame, or `None` when memory is exhausted.
    pub fn allocate(&mut self) -> Option<PhysFrame> {
        while let Some(region) = self.regions.get(self.region_index) {
            if region.kind != MemoryRegionKind::Usable {
                self.region_index += 1;
                self.next_address = None;
                continue;
            }

            let region_end = region.start + region.length;
            let candidate = match self.next_address {
                Some(address) => address,
                None => align_up(region.start, self.page_size)?,
            };

            if let Some(frame_end) = candidate.checked_add(self.page_size) {
                if frame_end <= region_end {
                    self.next_address = Some(frame_end);
                    return Some(PhysFrame { address: candidate });
                }
            }

            self.region_index += 1;
            self.next_address = None;
        }

        None
    }
}

fn align_up(address: u64, alignment: u64) -> Option<u64> {
    address
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
}

#[cfg(test)]
mod tests {
    use super::{FrameAllocator, FrameAllocatorError, MemoryRegion, MemoryRegionKind, PhysFrame};

    const KIB: u64 = 1024;

    fn region(start: u64, length: u64, kind: MemoryRegionKind) -> MemoryRegion {
        MemoryRegion {
            start,
            length,
            kind,
        }
    }

    #[test]
    fn aligns_usable_regions_to_page_size() {
        let regions = [region(1, 8192, MemoryRegionKind::Usable)];
        let mut allocator = FrameAllocator::new(&regions, 4096).unwrap();

        assert_eq!(allocator.allocate(), Some(PhysFrame { address: 4096 }));
        assert_eq!(allocator.allocate(), None);
    }

    #[test]
    fn skips_reserved_regions() {
        let regions = [
            region(0, 4096, MemoryRegionKind::Reserved),
            region(4096, 8192, MemoryRegionKind::Usable),
        ];
        let mut allocator = FrameAllocator::new(&regions, 4096).unwrap();

        assert_eq!(allocator.allocate(), Some(PhysFrame { address: 4096 }));
        assert_eq!(allocator.allocate(), Some(PhysFrame { address: 8192 }));
        assert_eq!(allocator.allocate(), None);
    }

    #[test]
    fn supports_architecture_specific_page_sizes() {
        let regions = [region(0, 32 * KIB, MemoryRegionKind::Usable)];
        let mut allocator = FrameAllocator::new(&regions, 16 * KIB).unwrap();

        assert_eq!(allocator.allocate(), Some(PhysFrame { address: 0 }));
        assert_eq!(allocator.allocate(), Some(PhysFrame { address: 16 * KIB }));
        assert_eq!(allocator.allocate(), None);
    }

    #[test]
    fn rejects_invalid_page_sizes_and_memory_maps() {
        let regions = [region(0, 4096, MemoryRegionKind::Usable)];
        assert_eq!(
            FrameAllocator::new(&regions, 3000).err(),
            Some(FrameAllocatorError::InvalidPageSize)
        );

        let overlapping = [
            region(0, 8192, MemoryRegionKind::Usable),
            region(4096, 4096, MemoryRegionKind::Reserved),
        ];
        assert_eq!(
            FrameAllocator::new(&overlapping, 4096).err(),
            Some(FrameAllocatorError::RegionsNotOrderedOrOverlapping)
        );

        let overflowing = [region(u64::MAX - 3, 8, MemoryRegionKind::Usable)];
        assert_eq!(
            FrameAllocator::new(&overflowing, 4096).err(),
            Some(FrameAllocatorError::RegionAddressOverflow)
        );
    }
}
