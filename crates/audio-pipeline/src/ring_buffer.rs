//! Lock-free Single-Producer Single-Consumer ring buffer
//!
//! Designed for inter-core communication in embedded systems.
//! No heap allocation - all storage is pre-allocated.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use portable_atomic::{AtomicUsize, Ordering};

/// Lock-free SPSC ring buffer
///
/// # Safety
/// This buffer is only safe for single-producer, single-consumer usage.
/// The producer should only call `write` and `available_write`.
/// The consumer should only call `read` and `available_read`.
pub struct RingBuffer<T, const N: usize> {
    buffer: UnsafeCell<[MaybeUninit<T>; N]>,
    head: AtomicUsize, // Write position (producer)
    tail: AtomicUsize, // Read position (consumer)
}

// Safety: RingBuffer is Sync because we use atomic operations for head/tail
// and the SPSC pattern ensures no data races on the buffer itself.
unsafe impl<T: Send, const N: usize> Sync for RingBuffer<T, N> {}
unsafe impl<T: Send, const N: usize> Send for RingBuffer<T, N> {}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    /// Create a new empty ring buffer
    ///
    /// N must be a power of 2 for efficient modulo operations.
    pub const fn new() -> Self {
        assert!(N > 0, "Buffer size must be > 0");
        assert!(N.is_power_of_two(), "Buffer size must be power of 2");

        Self {
            buffer: UnsafeCell::new(
                // Safety: MaybeUninit doesn't require initialization
                unsafe { MaybeUninit::uninit().assume_init() },
            ),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Number of items that can be read
    pub fn available_read(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head.wrapping_sub(tail)
    }

    /// Number of items that can be written
    pub fn available_write(&self) -> usize {
        N - 1 - self.available_read() // Leave one slot empty to distinguish full from empty
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.available_read() == 0
    }

    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.available_write() == 0
    }

    /// Write items to the buffer (producer only)
    ///
    /// Returns the number of items actually written.
    pub fn write(&self, data: &[T]) -> usize {
        let available = self.available_write();
        let to_write = data.len().min(available);

        if to_write == 0 {
            return 0;
        }

        let head = self.head.load(Ordering::Relaxed);

        // Safety: We're the only producer, and we're writing to slots
        // that are not being read (head to head + to_write).
        let buffer = unsafe { &mut *self.buffer.get() };

        // Bounded loop: to_write iterations
        for i in 0..to_write {
            let idx = (head + i) & (N - 1); // Fast modulo for power of 2
            buffer[idx] = MaybeUninit::new(data[i]);
        }

        // Publish the new head
        self.head
            .store(head.wrapping_add(to_write), Ordering::Release);

        to_write
    }

    /// Read items from the buffer (consumer only)
    ///
    /// Returns the number of items actually read.
    pub fn read(&self, buf: &mut [T]) -> usize {
        let available = self.available_read();
        let to_read = buf.len().min(available);

        if to_read == 0 {
            return 0;
        }

        let tail = self.tail.load(Ordering::Relaxed);

        // Safety: We're the only consumer, and we're reading from slots
        // that are not being written (tail to tail + to_read).
        let buffer = unsafe { &*self.buffer.get() };

        // Bounded loop: to_read iterations
        for i in 0..to_read {
            let idx = (tail + i) & (N - 1);
            // Safety: This slot was written by the producer
            buf[i] = unsafe { buffer[idx].assume_init() };
        }

        // Publish the new tail
        self.tail
            .store(tail.wrapping_add(to_read), Ordering::Release);

        to_read
    }

    /// Clear the buffer (both producer and consumer must be idle)
    pub fn clear(&self) {
        self.head.store(0, Ordering::Release);
        self.tail.store(0, Ordering::Release);
    }
}

impl<T: Copy, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer_empty() {
        let buffer: RingBuffer<u8, 16> = RingBuffer::new();
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
        assert_eq!(buffer.available_read(), 0);
        assert_eq!(buffer.available_write(), 15); // N-1 because one slot is sentinel
    }

    #[test]
    fn test_write_read_single() {
        let buffer: RingBuffer<u8, 16> = RingBuffer::new();

        let written = buffer.write(&[42]);
        assert_eq!(written, 1);
        assert_eq!(buffer.available_read(), 1);

        let mut out = [0u8; 1];
        let read = buffer.read(&mut out);
        assert_eq!(read, 1);
        assert_eq!(out[0], 42);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_write_read_multiple() {
        let buffer: RingBuffer<u8, 16> = RingBuffer::new();

        let data = [1, 2, 3, 4, 5];
        let written = buffer.write(&data);
        assert_eq!(written, 5);

        let mut out = [0u8; 5];
        let read = buffer.read(&mut out);
        assert_eq!(read, 5);
        assert_eq!(out, data);
    }

    #[test]
    fn test_write_full() {
        let buffer: RingBuffer<u8, 8> = RingBuffer::new();

        // Try to write more than capacity
        let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let written = buffer.write(&data);
        assert_eq!(written, 7); // N-1 = 7

        assert!(buffer.is_full());
        assert_eq!(buffer.available_write(), 0);
    }

    #[test]
    fn test_read_empty() {
        let buffer: RingBuffer<u8, 16> = RingBuffer::new();

        let mut out = [0u8; 5];
        let read = buffer.read(&mut out);
        assert_eq!(read, 0);
    }

    #[test]
    fn test_wrap_around() {
        let buffer: RingBuffer<u8, 8> = RingBuffer::new();

        // Fill partially
        buffer.write(&[1, 2, 3, 4, 5]);

        // Read some
        let mut out = [0u8; 3];
        buffer.read(&mut out);
        assert_eq!(out, [1, 2, 3]);

        // Write more (wraps around)
        buffer.write(&[6, 7, 8, 9, 10]);

        // Read all
        let mut out2 = [0u8; 7];
        let read = buffer.read(&mut out2);
        assert_eq!(read, 7);
        assert_eq!(out2, [4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_clear() {
        let buffer: RingBuffer<u8, 16> = RingBuffer::new();

        buffer.write(&[1, 2, 3, 4, 5]);
        assert!(!buffer.is_empty());

        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_i16_samples() {
        let buffer: RingBuffer<i16, 256> = RingBuffer::new();

        let samples: [i16; 128] = core::array::from_fn(|i| i as i16 * 100);
        let written = buffer.write(&samples);
        assert_eq!(written, 128);

        let mut out = [0i16; 128];
        let read = buffer.read(&mut out);
        assert_eq!(read, 128);
        assert_eq!(out, samples);
    }
}
