use std::{collections::BTreeMap, ffi::CStr, marker::PhantomData};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ParamTypeOffset {
    unk04: u32,
    param_type_offset: u32,
    unk_pad: [u8; 24],
}

#[repr(C)]
#[derive(Clone, Copy)]
union ParamTypeBlock {
    param_type_buf: [u8; 32],
    offset: ParamTypeOffset,
}
impl std::fmt::Debug for ParamTypeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParamTypeBlock")
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ParamFileHeader {
    strings_offset: u32,
    short_data_offset: u16,
    unk006: u16,
    paramdef_data_version: u16,
    row_count: u16,
    param_type_block: ParamTypeBlock,
    is_big_endian: u8,
    format_flags_2d: u8,
    format_flags_2e: u8,
    paramdef_format_version: u8,
}

impl ParamFileHeader {
    pub fn header_size(&self) -> usize {
        let f = self.format_flags_2d;
        if (f & 3) == 3 || (f & 4) != 0 {
            0x40
        }
        else {
            0x30
        }
    }

    pub fn row_count(&self) -> u16 {
        return self.row_count;
    }

    pub fn is_big_endian(&self) -> bool {
        return self.is_big_endian != 0;
    }

    pub fn is_unicode(&self) -> bool {
        return (self.format_flags_2e & 1) != 0;
    }

    pub fn is_64_bit(&self) -> bool {
        (self.format_flags_2d & 4) != 0
    }

    pub fn data_end_ofs(&self) -> usize {
        if (self.format_flags_2d & 0x80) != 0 {
            unsafe { self.param_type_block.offset }.param_type_offset as usize
        }
        else {
            self.strings_offset as usize
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromBytesError {
    InsufficientAlignment,
    BufferTooSmall,
    UnsupportedFile { is_big_endian: bool, is_64bit: bool },
    OutOfBoundsOffset,
    IntersectingData,
    UnsortedRowDescs,
    DuplicateIds,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ParamRowDescriptor {
    pub id: u32,
    pub data_offset: usize,
    pub name_offset: usize,
}

#[derive(Debug)]
pub struct ParamFile<'a> {
    data: *mut u8,
    file_size: usize,
    row_size: usize,
    header: &'a ParamFileHeader,
    row_descriptors: &'a [ParamRowDescriptor],
}

#[derive(Debug, Clone, Copy)]
pub struct Row<'a> {
    id: u32,
    data: &'a [u8],
}

#[derive(Debug)]
pub struct RowMut<'a> {
    id: u32,
    data: &'a mut [u8],
}

impl<'a> ParamFile<'a> {
    /// Creates param file from a mutable byte slice, without any sanity checks.
    ///
    /// # Safety:
    /// - The byte slice must be aligned to a usize multiple.
    /// - The byte slice must represent a valid param file with endianness and bitness corresponding
    ///   to the target platform.
    pub unsafe fn from_bytes_unchecked(data: &'a mut [u8]) -> Self {
        let header = &*(data.as_ptr() as usize as *const ParamFileHeader);
        let row_descriptors = std::slice::from_raw_parts(
            (data.as_ptr() as usize + header.header_size()) as *const ParamRowDescriptor,
            header.row_count as usize,
        );
        let row_size = match row_descriptors.len() {
            0 => 0, // Obviously not true, but there are no rows so doesn't matter :)
            1 => header.data_end_ofs() - row_descriptors[0].data_offset as usize,
            _ => (row_descriptors[1].data_offset - row_descriptors[0].data_offset) as usize,
        };
        Self {
            data: data.as_mut_ptr(),
            file_size: data.len(),
            row_size,
            header,
            row_descriptors,
        }
    }

    /// Creates a param file from a mutable byte slice, checking if it contains safe data **for the purposes of this API**.
    ///
    /// # Errors
    /// - If the slice is not aligned to a usize multiple, returns [`FromBytesError::InsufficientAlignment`].
    /// - If the slice is too small, returns [`FromBytesError::BufferTooSmall`].
    /// - If the param file is designed for a system with a different endianness
    ///   or bitness, returns [`FromBytesError::UnsupportedFile`].
    /// - If row descriptors are not sorted by unique IDs, returns [`FromBytesError::UnsortedRowDescs`].
    /// - If one of the offsets in the file goes out of bounds, returns [`FromBytesError::OutOfBoundsOffset`].
    /// - If two data regions intersect, returns [`FromBytesError::IntersectingData`].
    pub fn from_bytes(data: &'a mut [u8]) -> Result<Self, FromBytesError> {
        let addr = data.as_ptr() as usize;

        // Check alignment
        if (addr & std::mem::align_of::<usize>()) != 0 {
            return Err(FromBytesError::InsufficientAlignment);
        }
        // Ensure large enough for the header
        if data.len() < std::mem::size_of::<ParamFileHeader>() {
            return Err(FromBytesError::BufferTooSmall);
        }
        let header = unsafe { &*(addr as *const ParamFileHeader) };

        const EXPECTED_OFFSET_SZ: usize = std::mem::size_of::<usize>();
        let offset_sz = if header.is_64_bit() { 8 } else { 4 };

        #[cfg(target_endian = "little")]
        const BIG_ENDIAN: bool = false;
        #[cfg(target_endian = "big")]
        const BIG_ENDIAN: bool = true;

        // Ensure file endianness and bitness matches
        if header.is_big_endian() != BIG_ENDIAN || EXPECTED_OFFSET_SZ != offset_sz {
            return Err(FromBytesError::UnsupportedFile {
                is_big_endian: header.is_big_endian(),
                is_64bit: header.is_64_bit(),
            });
        }

        // Ensure enough space is available for all row descriptors
        let row_desc_sz = header.row_count as usize * std::mem::size_of::<ParamRowDescriptor>();
        if data.len() < header.header_size() + row_desc_sz {
            return Err(FromBytesError::BufferTooSmall);
        }
        let row_descriptors = unsafe {
            std::slice::from_raw_parts(
                (addr as usize + header.header_size()) as *const ParamRowDescriptor,
                header.row_count as usize,
            )
        };
        let row_size = match row_descriptors.len() {
            0 => 0, // Obviously not true, but there are no rows so doesn't matter :)
            1 => header.data_end_ofs() - row_descriptors[0].data_offset as usize,
            _ => (row_descriptors[1].data_offset - row_descriptors[0].data_offset) as usize,
        };

        // Check if row descriptors are strictly sorted by ID
        if !row_descriptors.windows(2).all(|p| p[0].id < p[1].id) {
            return Err(FromBytesError::UnsortedRowDescs);
        }

        // Collect all data blocks we might access in the file, and
        // make sure they (1) aren't out of bounds and (2) don't intersect other blocks
        let mut used_blocks: Vec<_> =
            row_descriptors.iter().map(|r| (r.data_offset, row_size)).collect();

        used_blocks.push((0usize, header.header_size() + row_desc_sz));
        used_blocks.push((header.data_end_ofs(), data.len() - header.data_end_ofs()));
        used_blocks.sort_by_key(|b| b.0);

        let mut last_block_end = 0;
        for (ofs, size) in used_blocks {
            if ofs < last_block_end {
                return Err(FromBytesError::IntersectingData);
            }
            last_block_end = ofs + size;
        }
        if last_block_end > data.len() {
            return Err(FromBytesError::OutOfBoundsOffset);
        }

        Ok(Self {
            data: data.as_mut_ptr(),
            file_size: data.len(),
            row_size,
            header,
            row_descriptors,
        })
    }

    pub fn row_size(&self) -> usize {
        self.row_size
    }

    pub fn header(&self) -> &ParamFileHeader {
        return unsafe { &*(self.data as usize as *const ParamFileHeader) };
    }

    pub fn row_descriptors(&self) -> &[ParamRowDescriptor] {
        &self.row_descriptors
    }

    pub fn rows(&self) -> impl Iterator<Item = Row<'_>> {
        self.row_descriptors.iter().map(|r| Row {
            id: r.id,
            data: unsafe {
                std::slice::from_raw_parts(self.data.add(r.data_offset), self.row_size)
            },
        })
    }

    pub fn rows_mut(&mut self) -> impl Iterator<Item = RowMut<'_>> {
        self.row_descriptors.iter().map(|r| RowMut {
            id: r.id,
            data: unsafe {
                std::slice::from_raw_parts_mut(self.data.add(r.data_offset), self.row_size)
            },
        })
    }

    pub fn get(&self, index: usize) -> Option<Row<'_>> {
        let r = self.row_descriptors.get(index)?;
        Some(Row {
            id: r.id,
            data: unsafe {
                std::slice::from_raw_parts(self.data.add(r.data_offset), self.row_size)
            },
        })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<RowMut<'_>> {
        let r = self.row_descriptors.get(index)?;
        Some(RowMut {
            id: r.id,
            data: unsafe {
                std::slice::from_raw_parts_mut(self.data.add(r.data_offset), self.row_size)
            },
        })
    }

    pub fn index_of(&self, row_id: u32) -> Option<usize> {
        self.row_descriptors.binary_search_by_key(&row_id, |r| r.id).ok()
    }

    pub fn by_id(&self, id: u32) -> Option<Row<'_>> {
        self.get(self.index_of(id)?)
    }

    pub fn by_id_mut(&mut self, id: u32) -> Option<RowMut<'_>> {
        self.get_mut(self.index_of(id)?)
    }
}

impl<'a> std::ops::Index<usize> for ParamFile<'a> {
    type Output = [u8];
    fn index(&self, index: usize) -> &Self::Output {
        let r = &self.row_descriptors[index];
        unsafe { std::slice::from_raw_parts(self.data.add(r.data_offset), self.row_size) }
    }
}

impl<'a> std::ops::IndexMut<usize> for ParamFile<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let r = &self.row_descriptors[index];
        unsafe { std::slice::from_raw_parts_mut(self.data.add(r.data_offset), self.row_size) }
    }
}
