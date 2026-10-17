pub const CACHE_SECTORS: usize = 16;

#[derive(Copy, Clone)]
pub struct CacheEntry {
    pub lba: u32,
    pub valid: bool,
}

pub struct LbaCache {
    pub entries: [CacheEntry; CACHE_SECTORS],
    pub data: [[u8; 512]; CACHE_SECTORS],
}

impl LbaCache {
    pub const fn new() -> Self {
        Self {
            entries: [CacheEntry {
                lba: 0xFFFF_FFFF,
                valid: false,
            }; CACHE_SECTORS],
            data: [[0u8; 512]; CACHE_SECTORS],
        }
    }

    #[inline]
    pub fn lookup(&self, lba: u32) -> Option<&[u8; 512]> {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.valid && entry.lba == lba {
                return Some(&self.data[i]);
            }
        }
        None
    }

    pub fn insert(&mut self, slot: usize, lba: u32, buf: &[u8; 512]) {
        if slot < CACHE_SECTORS {
            self.entries[slot] = CacheEntry { lba, valid: true };
            self.data[slot].copy_from_slice(buf);
        }
    }

    pub fn invalidate(&mut self, lba: u32) {
        for entry in self.entries.iter_mut() {
            if entry.valid && entry.lba == lba {
                entry.valid = false;
            }
        }
    }

    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.valid = false;
            entry.lba = 0xFFFF_FFFF;
        }
    }

    pub fn update(&mut self, lba: u32, buf: &[u8; 512]) {
        for (i, entry) in self.entries.iter_mut().enumerate() {
            if entry.valid && entry.lba == lba {
                self.data[i].copy_from_slice(buf);
            }
        }
    }
}
