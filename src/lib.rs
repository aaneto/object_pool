/// Represents a Pool object, where you can preallocate a grid
/// of Data preemptively.
///
/// The Pool has an internal linked list to free items, where
/// you can check the indexes and insert at those.
///
/// Usually, inserting should be pretty cheap and removing should be O(k)
/// where k is the number of free items.
///
/// There are ways to optimize this, I believe, but since this was just an
/// exercise, I just left this as is.
pub struct Pool {
    entries: Vec<PoolEntry>,
    first_free: Option<usize>,
}

impl Pool {
    /// Create a new Pool with N allocations.
    pub fn new(size: usize) -> Pool {
        let entries = (0..size).map(|idx| {
            if idx == 0 {
                PoolEntry::Empty(EmptyEntry::new(None, Some(idx + 1)))
            } else if idx == size - 1 {
                PoolEntry::Empty(EmptyEntry::new(Some(idx - 1), None))
            } else {
                PoolEntry::Empty(EmptyEntry::new(Some(idx - 1), Some(idx + 1)))
            }
        });
        Pool {
            entries: entries.collect(),
            first_free: Some(0),
        }
    }

    /// Get the index of the first free element.
    pub fn first_free(&self) -> Option<usize> {
        self.first_free
    }

    /// Get a particular entry.
    pub fn entry(&self, idx: usize) -> Option<&PoolEntry> {
        self.entries.get(idx)
    }

    /// Insert an item into the Pool, it will not crash on overflowing items, just ignore the request.
    pub fn set(&mut self, idx: usize, data: FilledEntry) -> Option<usize> {
        if idx >= self.entries.len() {
            return None;
        }

        let (prev, next) = match self.entries.get(idx).unwrap() {
            PoolEntry::Data(_) => {
                // There was data here before, noop on the empty linked-list.
                self.entries[idx] = PoolEntry::Data(data);
                return Some(idx);
            }
            PoolEntry::Empty(current_node) => (current_node.prev, current_node.next),
        };

        if Some(idx) == self.first_free {
            self.first_free = next;
        }

        if let Some(PoolEntry::Empty(prev_node)) = prev.map(|prev_id| {
            self.entries
                .get_mut(prev_id)
                .expect("Prev should always be valid.")
        }) {
            prev_node.next = next;
        }
        if let Some(PoolEntry::Empty(next_node)) = next.map(|next_id| {
            self.entries
                .get_mut(next_id)
                .expect("Next should always be valid.")
        }) {
            next_node.prev = prev;
        }

        self.entries[idx] = PoolEntry::Data(data);

        Some(idx)
    }

    /// Free a certain item from the Pool.
    pub fn free(&mut self, idx: usize) -> Option<usize> {
        if idx >= self.entries.len() {
            return None;
        }

        match self.entries.get_mut(idx).unwrap() {
            PoolEntry::Empty(_) => (),
            PoolEntry::Data(_) => {
                self.insert_on_free_list(idx);
            }
        };

        Some(idx)
    }

    /// To insert on the FreeList we either:
    /// - If this node will be inserted before first_free, replace first_free and add
    /// current first_free as new node next.
    ///
    /// - If this node will be inserted after first_free, it will replace a next on some
    /// node after (or including first_free).
    fn insert_on_free_list(&mut self, insert_index: usize) {
        let first_free_index = match self.first_free {
            Some(f_idx) => f_idx,
            None => {
                self.first_free = Some(insert_index);
                self.entries[insert_index] = PoolEntry::Empty(EmptyEntry::new(None, None));
                return;
            }
        };

        if insert_index == first_free_index {
            return;
        }

        if insert_index < first_free_index {
            if let Some(PoolEntry::Empty(empty)) = self.entries.get_mut(first_free_index) {
                empty.prev = Some(insert_index);
            }
            self.first_free = Some(insert_index);
            self.entries[insert_index] =
                PoolEntry::Empty(EmptyEntry::new(None, Some(first_free_index)));
            return;
        }

        let mut current_idx = first_free_index;
        while let Some(PoolEntry::Empty(empty)) = self.entries.get_mut(current_idx) {
            match empty.next {
                Some(n_idx) => {
                    if n_idx > insert_index {
                        break;
                    } else {
                        current_idx = n_idx;
                    }
                }
                None => break,
            }
        }

        let former_next =
            if let Some(PoolEntry::Empty(insert_node)) = self.entries.get_mut(current_idx) {
                let v = insert_node.next;
                insert_node.next = Some(insert_index);

                v
            } else {
                panic!("Should never happen");
            };

        self.entries[insert_index] =
            PoolEntry::Empty(EmptyEntry::new(Some(current_idx), former_next));
    }

    /// Get an iterator over the free or empty indexes.
    pub fn free_indexes(&self) -> PoolFreeIterator {
        PoolFreeIterator {
            pool: self,
            current_index: self.first_free,
        }
    }
}

/// Iterator over the free items of a Pool.
pub struct PoolFreeIterator<'s> {
    pool: &'s Pool,
    current_index: Option<usize>,
}

impl<'s> Iterator for PoolFreeIterator<'s> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let current_index = match self.current_index {
            Some(i) => i,
            None => return None,
        };
        match self.pool.entry(current_index) {
            Some(PoolEntry::Empty(entry)) => {
                let v = Some(current_index);
                if self.current_index == entry.next {
                    // Stuck in a loop.
                    return None;
                }
                self.current_index = entry.next;

                v
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum PoolEntry {
    Data(FilledEntry),
    Empty(EmptyEntry),
}

#[derive(Debug)]
pub struct FilledEntry {
    pub inner: Vec<u8>,
}

#[derive(Debug)]
pub struct EmptyEntry {
    pub prev: Option<usize>,
    pub next: Option<usize>,
}

impl EmptyEntry {
    pub fn new(prev: Option<usize>, next: Option<usize>) -> EmptyEntry {
        EmptyEntry { prev, next }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_empty() {
        let pool = Pool::new(12);

        assert_eq!(pool.first_free(), Some(0));
    }

    #[test]
    fn test_free_iterator() {
        let pool = Pool::new(12);
        let free_indexes: Vec<usize> = pool.free_indexes().collect();
        let all_indexes: Vec<usize> = (0..12).collect();

        assert_eq!(free_indexes, all_indexes);
    }

    #[test]
    fn test_insert() {
        let mut pool = Pool::new(10);
        pool.set(0, FilledEntry { inner: Vec::new() });
        pool.set(2, FilledEntry { inner: Vec::new() });
        pool.set(4, FilledEntry { inner: Vec::new() });
        pool.set(6, FilledEntry { inner: Vec::new() });

        let free_indexes: Vec<usize> = pool.free_indexes().collect();
        let expected_free: Vec<usize> = vec![1, 3, 5, 7, 8, 9];

        assert_eq!(free_indexes, expected_free);
    }

    #[test]
    fn test_insert2() {
        let mut pool = Pool::new(4);
        pool.set(0, FilledEntry { inner: Vec::new() });

        assert_eq!(pool.first_free, Some(1));
    }

    #[test]
    fn test_free() {
        let mut pool = Pool::new(4);
        pool.set(0, FilledEntry { inner: Vec::new() });
        pool.set(2, FilledEntry { inner: Vec::new() });
        pool.free(0);

        let free_indexes: Vec<usize> = pool.free_indexes().collect();
        let expected_free: Vec<usize> = vec![0, 1, 3];

        assert_eq!(free_indexes, expected_free);
        assert_eq!(pool.first_free, Some(0));
    }

    #[test]
    fn test_free2() {
        let mut pool = Pool::new(10);
        pool.set(0, FilledEntry { inner: Vec::new() });
        pool.set(2, FilledEntry { inner: Vec::new() });
        pool.set(4, FilledEntry { inner: Vec::new() });
        pool.set(6, FilledEntry { inner: Vec::new() });

        pool.free(0);
        pool.free(4);

        let free_indexes: Vec<usize> = pool.free_indexes().collect();
        let expected_free: Vec<usize> = vec![0, 1, 3, 4, 5, 7, 8, 9];

        assert_eq!(free_indexes, expected_free);
        assert_eq!(pool.first_free, Some(0));
    }
}
