//! Align and Stack spread the frames over the pool of the *CPUs* field (SP-015): the result does not
//! depend on its size.

mod common;

use common::{FRAMES, align_settings, point_settings, session, stack_settings};
use planets_session::Operation;
use std::sync::Mutex;

/// Runs Align and Stack with `cpus` threads; returns the ratings, the stacked picture and the progress of both.
fn run(cpus: u32) -> (String, Vec<u8>, Vec<(Operation, u32)>) {
    let session = session(&format!("parallel-{cpus}"));
    session.set_cpus(cpus).unwrap();
    let points = session.set_alignpoints(0, &point_settings()).unwrap();
    let seen = Mutex::new(Vec::new());
    let sink = |op, done, _, _| seen.lock().unwrap().push((op, done));
    let aligned = session.align(&align_settings(), &sink).unwrap();
    let mut settings = stack_settings(&points);
    settings.correct_geometry = true;
    settings.normalize = true;
    session.stack(&settings, &sink).unwrap();
    let rgba = session.stacked_image().unwrap().rgba().to_vec();
    (format!("{aligned:?}"), rgba, seen.into_inner().unwrap())
}

#[test]
fn one_cpu_gives_the_result_of_all_cores() {
    let (aligned_one, stack_one, progress_one) = run(1);
    let expected_progress: Vec<_> = (1..=FRAMES)
        .map(|done| (Operation::Align, done))
        .chain((1..=FRAMES).map(|done| (Operation::Stack, done)))
        .collect();
    assert_eq!(progress_one, expected_progress);
    for cpus in [2, 3, u32::MAX] {
        let (aligned, stack, progress) = run(cpus);
        assert_eq!(aligned, aligned_one, "{cpus} CPUs: Align differs");
        assert!(stack == stack_one, "{cpus} CPUs: the stack differs");
        assert_eq!(progress, expected_progress, "{cpus} CPUs: progress");
    }
}
