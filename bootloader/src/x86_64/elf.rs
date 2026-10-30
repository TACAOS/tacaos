//! ELF64 executable validation for the x86_64 boot path.

const ELF_HEADER_SIZE: usize = 64;
const PROGRAM_HEADER_SIZE: usize = 56;
const MAX_PROGRAM_HEADERS: usize = 32;
const MAX_LOAD_SEGMENTS: usize = 16;
const PAGE_SIZE: u64 = 4096;
const PT_LOAD: u32 = 1;
const PF_X: u32 = 1;

/// Errors encountered while validating an x86_64 ELF kernel image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElfError {
    InvalidHeader,
    UnsupportedFormat,
    InvalidProgramHeader,
    TooManySegments,
    NoLoadSegments,
    EntryNotExecutable,
    SegmentsOverlap,
}

/// A validated loadable segment in an ELF64 image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadSegment {
    pub file_offset: u64,
    pub virtual_address: u64,
    pub physical_address: u64,
    pub file_size: u64,
    pub memory_size: u64,
    pub flags: u32,
}

/// A validated static x86_64 ELF executable and its loadable segments.
pub struct ElfImage {
    entry: u64,
    segments: [Option<LoadSegment>; MAX_LOAD_SEGMENTS],
    segment_count: usize,
}

impl ElfImage {
    /// Validates an ELF64 little-endian x86_64 executable from its file bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, ElfError> {
        if bytes.get(0..4) != Some(b"\x7fELF") {
            return Err(ElfError::InvalidHeader);
        }
        if bytes.get(4) != Some(&2)
            || bytes.get(5) != Some(&1)
            || bytes.get(6) != Some(&1)
            || read_u16(bytes, 16) != Some(2)
            || read_u16(bytes, 18) != Some(62)
            || read_u32(bytes, 20) != Some(1)
        {
            return Err(ElfError::UnsupportedFormat);
        }
        if bytes.len() < ELF_HEADER_SIZE
            || read_u16(bytes, 52) != Some(ELF_HEADER_SIZE as u16)
            || read_u16(bytes, 54) != Some(PROGRAM_HEADER_SIZE as u16)
        {
            return Err(ElfError::InvalidHeader);
        }

        let entry = read_u64(bytes, 24).ok_or(ElfError::InvalidHeader)?;
        let program_offset = usize::try_from(read_u64(bytes, 32).ok_or(ElfError::InvalidHeader)?)
            .map_err(|_| ElfError::InvalidHeader)?;
        let program_count = usize::from(read_u16(bytes, 56).ok_or(ElfError::InvalidHeader)?);
        if program_count == 0 || program_count > MAX_PROGRAM_HEADERS {
            return Err(ElfError::InvalidHeader);
        }
        let program_table_size = program_count
            .checked_mul(PROGRAM_HEADER_SIZE)
            .ok_or(ElfError::InvalidHeader)?;
        let program_table_end = program_offset
            .checked_add(program_table_size)
            .ok_or(ElfError::InvalidHeader)?;
        if program_table_end > bytes.len() {
            return Err(ElfError::InvalidHeader);
        }

        let mut segments: [Option<LoadSegment>; MAX_LOAD_SEGMENTS] = [None; MAX_LOAD_SEGMENTS];
        let mut segment_count = 0;
        let mut executable_entry = false;

        for index in 0..program_count {
            let offset = program_offset + index * PROGRAM_HEADER_SIZE;
            if read_u32(bytes, offset) != Some(PT_LOAD) {
                continue;
            }
            if segment_count == MAX_LOAD_SEGMENTS {
                return Err(ElfError::TooManySegments);
            }

            let segment = LoadSegment {
                flags: read_u32(bytes, offset + 4).ok_or(ElfError::InvalidProgramHeader)?,
                file_offset: read_u64(bytes, offset + 8).ok_or(ElfError::InvalidProgramHeader)?,
                virtual_address: read_u64(bytes, offset + 16)
                    .ok_or(ElfError::InvalidProgramHeader)?,
                physical_address: read_u64(bytes, offset + 24)
                    .ok_or(ElfError::InvalidProgramHeader)?,
                file_size: read_u64(bytes, offset + 32).ok_or(ElfError::InvalidProgramHeader)?,
                memory_size: read_u64(bytes, offset + 40).ok_or(ElfError::InvalidProgramHeader)?,
            };
            let alignment = read_u64(bytes, offset + 48).ok_or(ElfError::InvalidProgramHeader)?;
            let file_end = segment
                .file_offset
                .checked_add(segment.file_size)
                .ok_or(ElfError::InvalidProgramHeader)?;
            let memory_end = segment
                .virtual_address
                .checked_add(segment.memory_size)
                .ok_or(ElfError::InvalidProgramHeader)?;

            if segment.memory_size == 0
                || segment.file_size > segment.memory_size
                || file_end > bytes.len() as u64
                || segment.virtual_address != segment.physical_address
                || segment.virtual_address % PAGE_SIZE != 0
                || alignment > 1
                    && (!alignment.is_power_of_two()
                        || segment.file_offset % alignment != segment.virtual_address % alignment)
            {
                return Err(ElfError::InvalidProgramHeader);
            }

            let allocation_end = align_up(memory_end).ok_or(ElfError::InvalidProgramHeader)?;
            for previous in segments[..segment_count].iter().flatten() {
                let previous_end = align_up(
                    previous
                        .virtual_address
                        .checked_add(previous.memory_size)
                        .ok_or(ElfError::InvalidProgramHeader)?,
                )
                .ok_or(ElfError::InvalidProgramHeader)?;
                if segment.virtual_address < previous_end
                    && previous.virtual_address < allocation_end
                {
                    return Err(ElfError::SegmentsOverlap);
                }
            }

            executable_entry |=
                segment.flags & PF_X != 0 && entry >= segment.virtual_address && entry < memory_end;
            segments[segment_count] = Some(segment);
            segment_count += 1;
        }

        if segment_count == 0 {
            return Err(ElfError::NoLoadSegments);
        }
        if !executable_entry {
            return Err(ElfError::EntryNotExecutable);
        }

        Ok(Self {
            entry,
            segments,
            segment_count,
        })
    }

    /// Returns the executable entry address.
    pub const fn entry(&self) -> u64 {
        self.entry
    }

    /// Returns the validated loadable segments in program-header order.
    pub fn segments(&self) -> impl Iterator<Item = LoadSegment> + '_ {
        self.segments[..self.segment_count]
            .iter()
            .filter_map(|segment| *segment)
    }
}

fn align_up(address: u64) -> Option<u64> {
    address
        .checked_add(PAGE_SIZE - 1)
        .map(|value| value & !(PAGE_SIZE - 1))
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?,
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset.checked_add(8)?)?.try_into().ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{ElfError, ElfImage, PROGRAM_HEADER_SIZE};
    use std::vec;
    use std::vec::Vec;

    const PROGRAM_HEADER_OFFSET: usize = 64;
    const SEGMENT_OFFSET: usize = 0x1000;

    fn fixture() -> Vec<u8> {
        let mut bytes = vec![0; SEGMENT_OFFSET + 1];
        bytes[0..4].copy_from_slice(b"\x7fELF");
        bytes[4] = 2;
        bytes[5] = 1;
        bytes[6] = 1;
        put_u16(&mut bytes, 16, 2);
        put_u16(&mut bytes, 18, 62);
        put_u32(&mut bytes, 20, 1);
        put_u64(&mut bytes, 24, 0x10_0000);
        put_u64(&mut bytes, 32, PROGRAM_HEADER_OFFSET as u64);
        put_u16(&mut bytes, 52, 64);
        put_u16(&mut bytes, 54, PROGRAM_HEADER_SIZE as u16);
        put_u16(&mut bytes, 56, 1);
        put_u32(&mut bytes, PROGRAM_HEADER_OFFSET, 1);
        put_u32(&mut bytes, PROGRAM_HEADER_OFFSET + 4, 5);
        put_u64(&mut bytes, PROGRAM_HEADER_OFFSET + 8, SEGMENT_OFFSET as u64);
        put_u64(&mut bytes, PROGRAM_HEADER_OFFSET + 16, 0x10_0000);
        put_u64(&mut bytes, PROGRAM_HEADER_OFFSET + 24, 0x10_0000);
        put_u64(&mut bytes, PROGRAM_HEADER_OFFSET + 32, 1);
        put_u64(&mut bytes, PROGRAM_HEADER_OFFSET + 40, 4096);
        put_u64(&mut bytes, PROGRAM_HEADER_OFFSET + 48, 4096);
        bytes[SEGMENT_OFFSET] = 0xc3;
        bytes
    }

    fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn accepts_a_valid_x86_64_kernel_image() {
        let bytes = fixture();
        let image = ElfImage::parse(&bytes).unwrap();

        assert_eq!(image.entry(), 0x10_0000);
        assert_eq!(image.segments().count(), 1);
    }

    #[test]
    fn rejects_non_x86_64_images() {
        let mut bytes = fixture();
        put_u16(&mut bytes, 18, 183);

        assert_eq!(
            ElfImage::parse(&bytes).err(),
            Some(ElfError::UnsupportedFormat)
        );
    }

    #[test]
    fn rejects_segments_outside_the_image() {
        let mut bytes = fixture();
        put_u64(&mut bytes, PROGRAM_HEADER_OFFSET + 8, u64::MAX);

        assert_eq!(
            ElfImage::parse(&bytes).err(),
            Some(ElfError::InvalidProgramHeader)
        );
    }

    #[test]
    fn requires_an_executable_entry_segment() {
        let mut bytes = fixture();
        put_u32(&mut bytes, PROGRAM_HEADER_OFFSET + 4, 4);

        assert_eq!(
            ElfImage::parse(&bytes).err(),
            Some(ElfError::EntryNotExecutable)
        );
    }
}
