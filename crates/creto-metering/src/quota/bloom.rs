//! Bloom filter for fast quota existence checking.
//!
//! Provides O(1) check to determine if a quota key might exist,
//! with configurable false positive rate (~1%).

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Configuration for bloom filter sizing.
#[derive(Debug, Clone)]
pub struct BloomConfig {
    /// Expected number of items (quotas).
    pub expected_items: usize,
    /// Target false positive rate (default: 0.01 = 1%).
    pub false_positive_rate: f64,
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            expected_items: 10_000,
            false_positive_rate: 0.01,
        }
    }
}

impl BloomConfig {
    /// Calculate optimal bit array size.
    /// Formula: m = -(n * ln(p)) / (ln(2)^2)
    pub fn calculate_bit_size(&self) -> usize {
        let n = self.expected_items as f64;
        let p = self.false_positive_rate;
        let m = -(n * p.ln()) / (2_f64.ln().powi(2));
        m.ceil() as usize
    }

    /// Calculate optimal number of hash functions.
    /// Formula: k = (m / n) * ln(2)
    pub fn calculate_num_hashes(&self) -> u32 {
        let m = self.calculate_bit_size() as f64;
        let n = self.expected_items as f64;
        let k = (m / n) * 2_f64.ln();
        k.ceil().max(1.0).min(8.0) as u32
    }

    /// Memory usage in bytes.
    pub fn memory_bytes(&self) -> usize {
        (self.calculate_bit_size() + 63) / 64 * 8
    }
}

/// Thread-safe bloom filter using atomic operations.
pub struct QuotaBloomFilter {
    bits: Vec<AtomicU64>,
    num_hashes: u32,
    bit_size: usize,
    item_count: AtomicUsize,
    hash_seeds: Vec<u64>,
}

impl QuotaBloomFilter {
    /// Create a new bloom filter with the given configuration.
    pub fn new(config: BloomConfig) -> Self {
        let bit_size = config.calculate_bit_size();
        let num_hashes = config.calculate_num_hashes();
        let num_words = (bit_size + 63) / 64;

        let bits: Vec<AtomicU64> = (0..num_words)
            .map(|_| AtomicU64::new(0))
            .collect();

        // Generate hash seeds using golden ratio
        let hash_seeds: Vec<u64> = (0..num_hashes)
            .map(|i| 0x517cc1b727220a95_u64.wrapping_mul(i as u64 + 1))
            .collect();

        Self {
            bits,
            num_hashes,
            bit_size,
            item_count: AtomicUsize::new(0),
            hash_seeds,
        }
    }

    /// Create with default configuration (10K items, 1% FPR).
    pub fn with_defaults() -> Self {
        Self::new(BloomConfig::default())
    }

    /// Insert a key into the bloom filter.
    pub fn insert(&self, key: &str) {
        for seed in &self.hash_seeds {
            let hash = self.hash_with_seed(key, *seed);
            let bit_index = (hash as usize) % self.bit_size;
            self.set_bit(bit_index);
        }
        self.item_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if a key might exist in the filter.
    /// Returns false = definitely not present.
    /// Returns true = possibly present (check cache/Redis).
    #[inline]
    pub fn might_contain(&self, key: &str) -> bool {
        for seed in &self.hash_seeds {
            let hash = self.hash_with_seed(key, *seed);
            let bit_index = (hash as usize) % self.bit_size;
            if !self.get_bit(bit_index) {
                return false;
            }
        }
        true
    }

    /// Hash key with seed using FNV-1a (fast, good distribution).
    #[inline]
    fn hash_with_seed(&self, key: &str, seed: u64) -> u64 {
        const FNV_PRIME: u64 = 0x00000100000001B3;
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;

        let mut hash = FNV_OFFSET ^ seed;
        for byte in key.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    #[inline]
    fn set_bit(&self, index: usize) {
        let word_index = index / 64;
        let bit_offset = index % 64;
        self.bits[word_index].fetch_or(1u64 << bit_offset, Ordering::Relaxed);
    }

    #[inline]
    fn get_bit(&self, index: usize) -> bool {
        let word_index = index / 64;
        let bit_offset = index % 64;
        (self.bits[word_index].load(Ordering::Relaxed) & (1u64 << bit_offset)) != 0
    }

    /// Get current item count.
    pub fn len(&self) -> usize {
        self.item_count.load(Ordering::Relaxed)
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Estimated false positive rate based on current fill.
    pub fn estimated_fpr(&self) -> f64 {
        let n = self.len() as f64;
        let m = self.bit_size as f64;
        let k = self.num_hashes as f64;
        (1.0 - (-k * n / m).exp()).powf(k)
    }

    /// Memory usage in bytes.
    pub fn memory_bytes(&self) -> usize {
        self.bits.len() * 8
    }

    /// Clear all bits (reset filter).
    pub fn clear(&self) {
        for word in &self.bits {
            word.store(0, Ordering::Relaxed);
        }
        self.item_count.store(0, Ordering::Relaxed);
    }
}

/// Quota key builder for consistent key formatting.
#[derive(Debug, Clone)]
pub struct QuotaKey {
    key: String,
}

impl QuotaKey {
    /// Create a quota key from components.
    pub fn new(
        org_id: &str,
        agent_id: &str,
        metric_code: &str,
        period: &str,
    ) -> Self {
        Self {
            key: format!("{}:{}:{}:{}", org_id, agent_id, metric_code, period),
        }
    }

    /// Get the key string.
    pub fn as_str(&self) -> &str {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_config_calculations() {
        let config = BloomConfig {
            expected_items: 10_000,
            false_positive_rate: 0.01,
        };

        let bit_size = config.calculate_bit_size();
        let num_hashes = config.calculate_num_hashes();
        let memory = config.memory_bytes();

        // For 10K items at 1% FPR, expect ~95K bits
        assert!(bit_size > 90_000 && bit_size < 100_000);
        assert_eq!(num_hashes, 7);
        assert!(memory < 20_000); // Less than 20KB
    }

    #[test]
    fn test_bloom_insert_and_check() {
        let bloom = QuotaBloomFilter::with_defaults();

        let key = QuotaKey::new("org1", "agent1", "api_calls", "daily");

        // Before insert, might_contain could be false
        bloom.insert(key.as_str());

        // After insert, must return true
        assert!(bloom.might_contain(key.as_str()));
        assert_eq!(bloom.len(), 1);
    }

    #[test]
    fn test_bloom_false_negative_rate() {
        let bloom = QuotaBloomFilter::with_defaults();

        // Insert 1000 keys
        for i in 0..1000 {
            let key = format!("org:agent:metric:{}", i);
            bloom.insert(&key);
        }

        // All inserted keys MUST return true (no false negatives)
        for i in 0..1000 {
            let key = format!("org:agent:metric:{}", i);
            assert!(bloom.might_contain(&key), "False negative for key {}", i);
        }
    }

    #[test]
    fn test_bloom_false_positive_rate() {
        let config = BloomConfig {
            expected_items: 1000,
            false_positive_rate: 0.01,
        };
        let bloom = QuotaBloomFilter::new(config);

        // Insert 1000 keys
        for i in 0..1000 {
            let key = format!("inserted:key:{}", i);
            bloom.insert(&key);
        }

        // Check 10000 keys that were NOT inserted
        let mut false_positives = 0;
        for i in 0..10000 {
            let key = format!("not_inserted:key:{}", i);
            if bloom.might_contain(&key) {
                false_positives += 1;
            }
        }

        // False positive rate should be around 1% (allow up to 3% for variance)
        let fpr = false_positives as f64 / 10000.0;
        assert!(fpr < 0.03, "FPR too high: {:.2}%", fpr * 100.0);
    }

    #[test]
    fn test_bloom_clear() {
        let bloom = QuotaBloomFilter::with_defaults();

        bloom.insert("key1");
        bloom.insert("key2");
        assert_eq!(bloom.len(), 2);

        bloom.clear();
        assert_eq!(bloom.len(), 0);
        // After clear, keys should not be found (with high probability)
    }

    #[test]
    fn test_quota_key_format() {
        let key = QuotaKey::new(
            "550e8400-e29b-41d4-a716-446655440000",
            "agent-123",
            "llm_tokens",
            "monthly",
        );

        assert_eq!(
            key.as_str(),
            "550e8400-e29b-41d4-a716-446655440000:agent-123:llm_tokens:monthly"
        );
    }
}
