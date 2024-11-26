use alloc::{boxed::Box, collections::VecDeque, sync::Arc};
use core::hash::Hash;
use hashbrown::HashMap;
use private::Range;

#[derive(Debug)]
pub enum Error {
    ValueLargerThanBuffer,
    ValueAlreadyPresent,
}

/// A FIFO cache for serializable objects with predictable size and almost no allocations at regime
/// and almost no wasted space.
///
/// The serialized cache requires an allocator.
///
/// Objects keys must be Hash of object values, in other words the same key maps to the same object.
/// Inserting the same key returns an error to discriminate the insertion case.
///
/// Almost no allocation means that fields indexes and insertions are obviously growing collections
/// but once the maximum number of objects is reached, at every insertion an element in these
/// collections is deleted, thus no extra growing is needed.
///
/// Almost no wasted space means that object are serialized one after the other so once the cache is
/// full only the latest bytes of the buffer are lost. Once the buffer is full, new serialized
/// object are inserted at the beginning, obviously overwriting oldest entries.
///
/// The average number of elements in the cache is `size(buffer)/average_size(object)`
///   
pub struct SliceCache<K: Hash + PartialEq + Eq + core::fmt::Debug> {
    /// Contains serialized objects one after the other, its size is defined at cache creation,
    /// once full, it starts again from the start
    buffer: Box<[u8]>,

    /// Pointer to free area in the buffer, it will be resetted to 0 once the buffer reach the end
    free_pointer: usize,

    /// Pointers to buffer of the serialized objects
    indexes: HashMap<Arc<K>, Range>,

    /// Order of the key inserted
    insertions: VecDeque<Arc<K>>,

    /// The cache is full, at least once it removed an older element to insert a new one.
    /// Obviously elements can still be inserted but they may remove older elements.
    full: bool,

    #[cfg(feature = "prometheus")]
    metric: prometheus::IntCounterVec,
}

mod private {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
    pub(crate) struct Range {
        begin: usize,
        end: usize, // must be >= begin
    }

    impl Range {
        pub fn from_begin_end(begin: usize, end: usize) -> Option<Self> {
            if end > begin {
                Some(Self { begin, end })
            } else {
                None
            }
        }

        pub fn from_begin_len(begin: usize, len: usize) -> Self {
            Self {
                begin,
                end: begin + len,
            }
        }

        pub fn begin(&self) -> usize {
            self.begin
        }
        pub fn end(&self) -> usize {
            self.end
        }

        pub fn overlaps(&self, other: &Range) -> bool {
            self.begin < other.end && other.begin < self.end

            // begin   end      b     e    OK                x
            // begin    b      end    e    KO   true         x  x
            // b       begin    e     end  KO   true         x  x
            // b        e     begin   end  OK                   x
        }
    }
}

impl<K: Hash + PartialEq + Eq + core::fmt::Debug> SliceCache<K> {
    /// Create the serialized cache with byte len equal to given `size`
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0u8; size].into_boxed_slice(),
            free_pointer: 0,
            indexes: HashMap::new(),
            insertions: VecDeque::new(),
            full: false,

            // TODO: metric name should be parametrized
            #[cfg(feature = "prometheus")]
            metric: prometheus::IntCounterVec::new(
                prometheus::Opts::new("slice_cache", "Counters for cache Hit/Miss"),
                &["event"],
            )
            .expect("statically defined"),
        }
    }

    /// Insert a value V in the cache, with key K
    /// returns the number of old entries removed
    pub fn insert<V: AsRef<[u8]>>(&mut self, key: K, value: &V) -> Result<usize, Error> {
        let value: &[u8] = value.as_ref();
        let mut removed = 0;

        if self.indexes.get(&key).is_some() {
            return Err(Error::ValueAlreadyPresent);
        }
        if value.len() > self.buffer.len() {
            return Err(Error::ValueLargerThanBuffer);
        }
        if value.len() + self.free_pointer > self.buffer.len() {
            // the element would not fit in the buffer, start again from the beginning,
            // but first remove any element in the buffer tail, otherwise inserted_range will not
            // overlap with the latest elements

            if let Some(range) = Range::from_begin_end(self.free_pointer, self.buffer.len()) {
                // we are removing only if the range is valid, it can happen `self.free_pointer == self.buffer.len()` and it that case we don't need to remove anything
                removed += self.remove_range(&range);
            }
            self.free_pointer = 0;
            self.full = true;
        }
        let begin = self.free_pointer;
        let end = begin + value.len();
        self.buffer[begin..end].copy_from_slice(value);
        self.free_pointer = end;

        let inserted_range = Range::from_begin_len(begin, value.len());
        let key = Arc::new(key);
        self.indexes.insert(key.clone(), inserted_range.clone());
        self.insertions.push_front(key);

        if self.insertions.len() > 1 {
            removed += self.remove_range(&inserted_range)
        }

        Ok(removed)
    }

    /// Get the value as slice at key `K` if exist in the cache, `None` otherwise
    pub fn get(&self, key: &K) -> Option<&[u8]> {
        let index = match self.indexes.get(key) {
            Some(val) => {
                #[cfg(feature = "prometheus")]
                self.metric.with_label_values(&["hit"]).inc();

                val
            }
            None => {
                #[cfg(feature = "prometheus")]
                self.metric.with_label_values(&["miss"]).inc();

                return None;
            }
        };

        Some(&self.buffer[index.begin()..index.end()])
    }

    /// Return wether the cache contains the given key
    pub fn contains(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    #[cfg(feature = "redb")]
    /// Get the value at key `K` if exist in the cache, `None` otherwise
    pub fn get_value<'a, V: redb::RedbValue>(&'a self, key: &K) -> Option<V::SelfType<'a>> {
        let slice = self.get(key)?;
        let value = V::from_bytes(slice);

        Some(value)
    }

    /// Return the number of elements contained in the cache
    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    /// Return the average size of the elements contained in the cache
    pub fn avg(&self) -> f64 {
        self.buffer.len() as f64 / self.indexes.len() as f64
    }

    /// Return wether the cache filled the inner buffer of serialized object and removed at least
    /// one older element, following inserted elements will likely remove older entries.
    pub fn full(&self) -> bool {
        self.full
    }

    fn remove_range(&mut self, range_to_remove: &Range) -> usize {
        let mut removed = 0;

        while let Some(back) = self.insertions.back() {
            let range = self
                .indexes
                .get(back)
                .expect("if in insertion, must be in indexes");
            if range_to_remove.overlaps(&range) {
                self.indexes.remove(back).expect("must be found");
                self.insertions.pop_back().expect("must be found");
                removed += 1;
            } else {
                break;
            }
        }
        removed
    }

    #[cfg(feature = "prometheus")]
    /// Register the inner metric for hit/cache in the prometheus registry
    pub fn register_metric(&self, r: &prometheus::Registry) -> Result<(), prometheus::Error> {
        r.register(Box::new(self.metric.clone()))
    }
}

#[cfg(test)]
mod tests {
    use hex_lit::hex;

    use crate::Parse;

    use super::*;

    #[test]
    fn insert_get() {
        let mut cache = SliceCache::new(10);

        let k1 = 0;
        let v1 = [1, 2];
        cache.insert(k1, &v1).unwrap();
        assert_eq!(cache.get(&k1), Some(&v1[..]));

        let k2 = 1;
        let v2 = [1, 2, 3];
        cache.insert(k2, &v2).unwrap();
        assert_eq!(cache.get(&k1), Some(&v1[..]));
        assert_eq!(cache.get(&k2), Some(&v2[..]));

        let k3 = 2;
        let v3 = [1, 2, 3, 4];
        cache.insert(k3, &v3).unwrap();
        assert_eq!(cache.get(&k1), Some(&v1[..]));
        assert_eq!(cache.get(&k2), Some(&v2[..]));
        assert_eq!(cache.get(&k3), Some(&v3[..]));
        println!("{:?}", cache.insertions);

        let k4 = 3;
        let v4 = [4, 5];
        cache.insert(k4, &v4).unwrap();
        assert_eq!(cache.get(&k1), None);
        assert_eq!(cache.get(&k2), Some(&v2[..]));
        assert_eq!(cache.get(&k3), Some(&v3[..]));
        assert_eq!(cache.get(&k4), Some(&v4[..]));
        println!("{:?}", cache.insertions);

        let k5 = 4;
        let v5 = [4, 5, 6, 7];
        cache.insert(k5, &v5).unwrap();
        assert_eq!(cache.get(&k1), None);
        assert_eq!(cache.get(&k2), None);
        assert_eq!(cache.get(&k3), None);
        assert_eq!(cache.get(&k4), Some(&v4[..]));
        assert_eq!(cache.get(&k5), Some(&v5[..]));
        println!("{:?}", cache.insertions);
    }

    #[cfg(feature = "prometheus")]
    #[test]
    fn prometheus() {
        use prometheus::Encoder;

        let r = prometheus::default_registry();

        let mut cache = SliceCache::new(10);
        cache.register_metric(&r).unwrap();

        let k1 = 0;
        let v1 = [1, 2];
        cache.insert(k1, &v1).unwrap();
        assert_eq!(cache.get(&k1), Some(&v1[..]));
        assert_eq!(cache.get(&1), None);

        let mut buffer = Vec::<u8>::new();
        let encoder = prometheus::TextEncoder::new();

        let metric_families = r.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        let result = format!("{}", String::from_utf8(buffer.clone()).unwrap());
        assert_eq!(result, "# HELP slice_cache Counters for cache Hit/Miss\n# TYPE slice_cache counter\nslice_cache{event=\"hit\"} 1\nslice_cache{event=\"miss\"} 1\n");
    }

    #[cfg(feature = "bitcoin")]
    #[test]
    fn with_transaction() {
        let segwit_tx = hex!("010000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff3603da1b0e00045503bd5704c7dd8a0d0ced13bb5785010800000000000a636b706f6f6c122f4e696e6a61506f6f6c2f5345475749542fffffffff02b4e5a212000000001976a914876fbb82ec05caa6af7a3b5e5a983aae6c6cc6d688ac0000000000000000266a24aa21a9edf91c46b49eb8a29089980f02ee6b57e7d63d33b18b4fddac2bcd7db2a39837040120000000000000000000000000000000000000000000000000000000000000000000000000");
        let tx = crate::bsl::Transaction::parse(&segwit_tx[..])
            .unwrap()
            .parsed_owned();
        let txid = tx.txid();

        let mut cache: SliceCache<_> = SliceCache::new(100_000);
        cache.insert(txid.clone(), &tx).unwrap();
        let val = cache.get(&txid).unwrap();

        assert_eq!(val, segwit_tx);
    }

    #[cfg(feature = "bitcoin")]
    #[test]
    fn with_transactions() {
        use bitcoin::consensus::Decodable;
        use bitcoin_test_data::blocks::mainnet_702861;
        use std::collections::HashMap;

        let block_slice = mainnet_702861();
        let block = bitcoin::Block::consensus_decode(&mut &block_slice[..]).unwrap();
        let txs: HashMap<_, _> = block
            .txdata
            .into_iter()
            .map(|tx| (tx.compute_txid(), bitcoin::consensus::serialize(&tx)))
            .collect();

        let cache_size = 600_000;
        let mut cache: SliceCache<_> = SliceCache::new(cache_size);

        let mut bytes_written = 0;
        let mut total_removed = 0;
        let mut inserted = vec![];
        for (txid, tx) in txs.iter() {
            let removed = cache.insert(txid.clone(), tx).unwrap();
            total_removed += removed;
            inserted.push(txid);
            bytes_written += tx.len();
            if bytes_written < cache_size {
                assert_eq!(removed, 0);
            }

            for inner_txid in inserted.iter().skip(total_removed) {
                let from_cache = cache.get(inner_txid).unwrap();
                let expected = txs.get(*inner_txid).unwrap();
                assert_eq!(from_cache, expected);
            }
        }
    }

    #[cfg(all(feature = "bitcoin", feature = "redb"))]
    #[test]
    fn with_transaction_value() {
        use crate::bsl::Transaction;

        let segwit_tx = hex!("010000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff3603da1b0e00045503bd5704c7dd8a0d0ced13bb5785010800000000000a636b706f6f6c122f4e696e6a61506f6f6c2f5345475749542fffffffff02b4e5a212000000001976a914876fbb82ec05caa6af7a3b5e5a983aae6c6cc6d688ac0000000000000000266a24aa21a9edf91c46b49eb8a29089980f02ee6b57e7d63d33b18b4fddac2bcd7db2a39837040120000000000000000000000000000000000000000000000000000000000000000000000000");
        let tx = Transaction::parse(&segwit_tx[..]).unwrap().parsed_owned();
        let txid = tx.txid();

        let mut cache: SliceCache<_> = SliceCache::new(100_000);
        cache.insert(txid.clone(), &tx).unwrap();
        let val = cache.get_value::<Transaction>(&txid).unwrap();

        assert_eq!(val.as_ref(), segwit_tx);
    }

    #[test]
    fn insert_when_buffer_exactly_full() {
        let mut cache = SliceCache::new(10);

        let k1 = 0;
        let v1 = [0; 10usize];
        cache.insert(k1, &v1).unwrap();

        let k2 = 1;
        let v2 = [0];
        cache.insert(k2, &v2).unwrap();
    }
}
