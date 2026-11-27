//! Frames in batches, for the jobs that register or stack a whole recording.
//!
//! A recording is read in order by one thread at a time (`read` may borrow a file). The frames come in
//! batches of about one frame per thread of the pool: while a batch is worked on, the next one is read,
//! so memory holds at most two batches, however long the recording is. What a batch gives the caller is in
//! the order of the recording, so a job that adds the frames of a batch one after the other does exactly
//! what it did frame by frame.

use crate::{Image, align::Control};

/// The most memory two batches (the one worked on and the one being read) may take together, in bytes.
const BATCH_BYTES: usize = 1 << 30;

/// Why [`for_each_batch`] stopped early.
pub(crate) enum Stop<E> {
    /// Reading a frame failed.
    Read(E),
    /// `process` asked to stop.
    Cancelled,
}

/// Frames per batch: one per thread of the current pool, fewer for very large frames so that two
/// batches stay within [`BATCH_BYTES`]. `frame_bytes` is the size of one frame in memory.
pub(crate) fn batch_size(frame_bytes: usize) -> usize {
    let by_memory = BATCH_BYTES / (2 * frame_bytes.max(1));
    rayon::current_num_threads().min(by_memory).max(1)
}

/// Reads the frames `indices` in order, `size` at a time, and hands every batch to `process` while the
/// next one is read in parallel to it. `process` answers whether to go on; a [`Control::Cancel`] ends the
/// job after the batch it came from. A frame that cannot be read ends it after the batches before it.
pub(crate) fn for_each_batch<E: Send>(
    indices: &[u32],
    size: usize,
    mut read: impl FnMut(u32) -> Result<Image, E> + Send,
    mut process: impl FnMut(&[Image]) -> Control + Send,
) -> Result<(), Stop<E>> {
    let mut read_batch = |part: &[u32]| {
        part.iter()
            .map(|&index| read(index))
            .collect::<Result<Vec<Image>, E>>()
    };
    let mut parts = indices.chunks(size.max(1));
    let Some(first) = parts.next() else {
        return Ok(());
    };
    let mut current = read_batch(first).map_err(Stop::Read)?;
    loop {
        let next = parts.next();
        let (control, read_next) = rayon::join(|| process(&current), || next.map(&mut read_batch));
        if control == Control::Cancel {
            return Err(Stop::Cancelled);
        }
        match read_next {
            None => return Ok(()),
            Some(frames) => current = frames.map_err(Stop::Read)?,
        }
    }
}
