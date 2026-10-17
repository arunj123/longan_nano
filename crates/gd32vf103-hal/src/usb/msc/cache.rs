pub const CACHE_SECTORS: usize = 16;

#[derive(Copy, Clone)]
pub struct CacheEntry {
    pub lba: u32,
    pub valid: bool,
}

#[cfg(not(test))]
static mut CACHE_DATA: [[u8; 512]; CACHE_SECTORS] = [[0u8; 512]; CACHE_SECTORS];

pub struct LbaCache {
    pub entries: [CacheEntry; CACHE_SECTORS],
    pub next_insert_slot: usize,
    #[cfg(test)]
    pub data: [[u8; 512]; CACHE_SECTORS],
}

impl LbaCache {
    pub const fn new() -> Self {
        Self {
            entries: [CacheEntry {
                lba: 0xFFFF_FFFF,
                valid: false,
            }; CACHE_SECTORS],
            next_insert_slot: 0,
            #[cfg(test)]
            data: [[0u8; 512]; CACHE_SECTORS],
        }
    }

    #[inline]
    pub fn lookup(&self, lba: u32) -> Option<&[u8; 512]> {
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.valid && entry.lba == lba {
                #[cfg(not(test))]
                unsafe {
                    return Some(&CACHE_DATA[i]);
                }
                #[cfg(test)]
                return Some(&self.data[i]);
            }
        }
        None
    }

    pub fn insert(&mut self, slot: usize, lba: u32, buf: &[u8; 512]) {
        if slot < CACHE_SECTORS {
            self.entries[slot] = CacheEntry { lba, valid: true };
            #[cfg(not(test))]
            unsafe {
                CACHE_DATA[slot].copy_from_slice(buf);
            }
            #[cfg(test)]
            self.data[slot].copy_from_slice(buf);
        }
    }

    /// Insert an LBA sector using a round-robin eviction policy across the cache slots.
    pub fn insert_lba(&mut self, lba: u32, buf: &[u8; 512]) {
        let slot = self.next_insert_slot;
        self.insert(slot, lba, buf);
        self.next_insert_slot = (slot + 1) % CACHE_SECTORS;
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
                #[cfg(not(test))]
                unsafe {
                    CACHE_DATA[i].copy_from_slice(buf);
                }
                #[cfg(test)]
                self.data[i].copy_from_slice(buf);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lba_cache_operations() {
        let mut cache = LbaCache::new();
        assert_eq!(cache.entries.len(), CACHE_SECTORS);
        assert!(cache.lookup(100).is_none());

        let mut b1 = [0xAAu8; 512];
        b1[0] = 0x12;
        cache.insert(0, 100, &b1);

        let hit = cache.lookup(100);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap()[0], 0x12);
        assert_eq!(hit.unwrap()[1], 0xAA);

        // Update
        let mut b2 = [0xBBu8; 512];
        b2[0] = 0x34;
        cache.update(100, &b2);
        let hit2 = cache.lookup(100);
        assert!(hit2.is_some());
        assert_eq!(hit2.unwrap()[0], 0x34);
        assert_eq!(hit2.unwrap()[1], 0xBB);

        // Invalidate
        cache.invalidate(100);
        assert!(cache.lookup(100).is_none());

        // Out-of-bounds slot is ignored safely
        cache.insert(CACHE_SECTORS, 200, &b1);
        assert!(cache.lookup(200).is_none());
    }

    #[test]
    fn test_lba_cache_insert_lba_round_robin() {
        let mut cache = LbaCache::new();
        let mut buf = [0u8; 512];

        // Insert 16 entries filling the cache
        for i in 0..CACHE_SECTORS {
            buf[0] = i as u8;
            cache.insert_lba(i as u32, &buf);
        }

        // Verify all 16 are resident
        for i in 0..CACHE_SECTORS {
            let hit = cache.lookup(i as u32);
            assert!(hit.is_some());
            assert_eq!(hit.unwrap()[0], i as u8);
        }

        // Next insert should evict slot 0 (LBA 0)
        buf[0] = 99;
        cache.insert_lba(1000, &buf);
        assert!(cache.lookup(0).is_none());
        assert!(cache.lookup(1000).is_some());
        assert_eq!(cache.lookup(1000).unwrap()[0], 99);
        assert!(cache.lookup(1).is_some());
    }
}

