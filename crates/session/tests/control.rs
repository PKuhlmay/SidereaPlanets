//! Cancel, Pause and the *CPUs* field of the session.

mod common;

use common::{FRAMES, align_settings, processed, session, stack_settings};
use planets_core::wavelet::ColourMode;
use planets_session::{
    ImageSource, NoProgress, Operation, SequenceSettings, Session, SessionError, logical_cores,
    wavelet_default_settings,
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

// ---- CPUs ----

#[test]
fn the_default_is_all_logical_cores() {
    let session = Session::new().unwrap();
    assert_eq!(session.cpu_count(), logical_cores());
    assert_eq!(session.cpus(), logical_cores());
}

#[test]
fn the_cpus_field_is_clamped_to_one_up_to_the_cores() {
    let session = Session::new().unwrap();
    assert_eq!(session.set_cpus(0), Ok(1));
    assert_eq!(session.cpus(), 1);
    assert_eq!(session.set_cpus(u32::MAX), Ok(logical_cores()));
    assert_eq!(session.cpus(), logical_cores());
    assert_eq!(session.set_cpus(1), Ok(1));
    assert_eq!(session.cpus(), 1);
}

#[test]
fn a_run_uses_the_pool_of_the_cpus_field() {
    let session = session("cpus-run");
    session
        .set_alignpoints(0, &common::point_settings())
        .unwrap();
    session.set_cpus(1).unwrap();
    let seen = Mutex::new(Vec::new());
    let sink = |_, _, _, _| seen.lock().unwrap().push(rayon::current_num_threads());
    session.align(&align_settings(), &sink).unwrap();
    // The sink is called from inside the pool, one entry per frame.
    assert_eq!(*seen.lock().unwrap(), vec![1; FRAMES as usize]);
}

// ---- Cancel ----

/// A sink that counts its calls and runs `then` after each.
fn counting<'a>(
    calls: &'a AtomicU32,
    then: impl Fn(u32) + Send + Sync + 'a,
) -> impl Fn(Operation, u32, u32, u64) + Send + Sync + 'a {
    move |_, done, _, _| {
        calls.fetch_add(1, Ordering::Relaxed);
        then(done);
    }
}

#[test]
fn cancel_stops_align_within_the_frame_that_is_running() {
    let session = session("cancel-align");
    session
        .set_alignpoints(0, &common::point_settings())
        .unwrap();
    let calls = AtomicU32::new(0);
    let sink = counting(&calls, |done| {
        if done == 2 {
            session.cancel(Operation::Align);
        }
    });
    let err = session.align(&align_settings(), &sink).unwrap_err();
    assert_eq!(err, SessionError::Cancelled);
    assert_eq!(calls.load(Ordering::Relaxed), 2, "no frame after Cancel");
}

#[test]
fn a_cancel_from_before_the_run_does_not_stop_the_next_one() {
    let session = session("stale-cancel");
    session
        .set_alignpoints(0, &common::point_settings())
        .unwrap();
    session.cancel(Operation::Align);
    assert!(session.align(&align_settings(), &NoProgress).is_ok());
}

#[test]
fn cancel_only_stops_the_operation_it_names() {
    let session = session("cancel-other");
    session
        .set_alignpoints(0, &common::point_settings())
        .unwrap();
    let sink = |_, _, _, _| {
        session.cancel(Operation::Wavelet);
        session.cancel(Operation::Stack);
    };
    assert!(session.align(&align_settings(), &sink).is_ok());
}

#[test]
fn a_cancelled_stack_leaves_the_earlier_stack_in_place() {
    let (session, points) = processed("cancel-stack");
    let before = session.stacked_image().unwrap();
    let sink = |_, _, _, _| session.cancel(Operation::Stack);
    let err = session.stack(&stack_settings(&points), &sink).unwrap_err();
    assert_eq!(err, SessionError::Cancelled);
    assert_eq!(session.stacked_image().unwrap(), before);
    assert!(session.require_image(ImageSource::Wavelet).is_ok());
}

#[test]
fn a_cancelled_do_all_leaves_the_earlier_result_in_place() {
    let (session, _) = processed("cancel-wavelet");
    let layers = AtomicU32::new(0);
    let sink = counting(&layers, |_| session.cancel(Operation::Wavelet));
    let err = session
        .wavelet_do_all(&wavelet_default_settings(), ColourMode::Colour, &sink)
        .unwrap_err();
    assert_eq!(err, SessionError::Cancelled);
    assert_eq!(layers.load(Ordering::Relaxed), 1);
    assert!(session.require_image(ImageSource::Wavelet).is_ok());
}

fn video(
    name: &str,
    points: &planets_session::Alignpoints,
) -> (SequenceSettings, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("planets-session-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let settings = SequenceSettings {
        stack: stack_settings(points),
        framerate: 25,
        sort_on_quality: false,
        average_frames: 2,
    };
    (settings, dir.join("out.avi"))
}

#[test]
fn save_aligned_and_save_integrated_share_cancel_with_stack() {
    let (session, points) = processed("cancel-sequence");
    let (settings, path) = video("cancel-sequence-out", &points);
    // Cancel arrives as *Stack* (the window sends one Cancel for all three), and it stops both videos.
    let sink = |_, _, _, _| session.cancel(Operation::Stack);
    let err = session.save_aligned(&path, &settings, &sink).unwrap_err();
    assert_eq!(err, SessionError::Cancelled);
    assert!(!path.exists(), "a cancelled video leaves no file");
    let err = session
        .save_integrated(&path, &settings, &sink)
        .unwrap_err();
    assert_eq!(err, SessionError::Cancelled);
    assert!(!path.exists());
    // Without a Cancel the same call writes the video.
    let saved = session.save_aligned(&path, &settings, &NoProgress).unwrap();
    assert_eq!(saved.frames, FRAMES);
    assert!(path.exists());
}

#[test]
fn cancel_of_the_sequence_stops_save_aligned() {
    let (session, points) = processed("cancel-sequence-op");
    let (settings, path) = video("cancel-sequence-op-out", &points);
    let sink = |_, _, _, _| session.cancel(Operation::Sequence);
    let err = session.save_aligned(&path, &settings, &sink).unwrap_err();
    assert_eq!(err, SessionError::Cancelled);
    assert!(!path.exists());
}

#[test]
fn save_aligned_reports_progress_as_a_sequence() {
    let (session, points) = processed("sequence-progress");
    let (settings, path) = video("sequence-progress-out", &points);
    let seen = Mutex::new(Vec::new());
    let sink = |op, done, total, _| seen.lock().unwrap().push((op, done, total));
    session.save_aligned(&path, &settings, &sink).unwrap();
    let expected: Vec<_> = (1..=FRAMES)
        .map(|done| (Operation::Sequence, done, FRAMES))
        .collect();
    assert_eq!(*seen.lock().unwrap(), expected);
}

// ---- Pause ----

#[test]
fn pause_holds_align_until_it_is_resumed() {
    let session = Arc::new(session("pause-align"));
    session
        .set_alignpoints(0, &common::point_settings())
        .unwrap();
    let calls = Arc::new(AtomicU32::new(0));
    let worker = {
        let (session, calls) = (session.clone(), calls.clone());
        std::thread::spawn(move || {
            let sink = |_, done, _, _| {
                calls.fetch_add(1, Ordering::Relaxed);
                if done == 1 {
                    session.pause(Operation::Align, true);
                }
            };
            session.align(&align_settings(), &sink)
        })
    };
    std::thread::sleep(Duration::from_millis(400));
    assert!(!worker.is_finished(), "the run waits while paused");
    assert_eq!(calls.load(Ordering::Relaxed), 1, "no progress while paused");
    session.pause(Operation::Align, false);
    let aligned = worker.join().unwrap().unwrap();
    assert_eq!(aligned.frames.len(), FRAMES as usize);
    assert_eq!(calls.load(Ordering::Relaxed), FRAMES);
}

#[test]
fn cancel_ends_a_pause() {
    let session = Arc::new(session("pause-cancel"));
    session
        .set_alignpoints(0, &common::point_settings())
        .unwrap();
    let worker = {
        let session = session.clone();
        std::thread::spawn(move || {
            let sink = |_, done, _, _| {
                if done == 1 {
                    session.pause(Operation::Align, true);
                }
            };
            session.align(&align_settings(), &sink)
        })
    };
    std::thread::sleep(Duration::from_millis(300));
    assert!(!worker.is_finished());
    session.cancel(Operation::Align);
    assert_eq!(worker.join().unwrap().unwrap_err(), SessionError::Cancelled);
}

#[test]
fn a_pause_from_before_the_run_does_not_hold_the_next_one() {
    let session = Arc::new(session("stale-pause"));
    session
        .set_alignpoints(0, &common::point_settings())
        .unwrap();
    session.pause(Operation::Align, true);
    let worker = {
        let session = session.clone();
        std::thread::spawn(move || session.align(&align_settings(), &NoProgress))
    };
    std::thread::sleep(Duration::from_millis(500));
    // A run that kept the old pause would still be waiting here.
    assert!(
        worker.is_finished(),
        "Align waits for a pause from before it started"
    );
    assert!(worker.join().unwrap().is_ok());
}

#[test]
fn pause_only_holds_the_operation_it_names() {
    let session = Arc::new(session("pause-other"));
    session
        .set_alignpoints(0, &common::point_settings())
        .unwrap();
    let worker = {
        let session = session.clone();
        std::thread::spawn(move || {
            let sink = |_, _, _, _| {
                session.pause(Operation::Wavelet, true);
                session.pause(Operation::Stack, true);
            };
            session.align(&align_settings(), &sink)
        })
    };
    std::thread::sleep(Duration::from_millis(500));
    // A run held by the wrong flag would still be waiting here.
    assert!(
        worker.is_finished(),
        "Align is held by the pause of another operation"
    );
    assert!(worker.join().unwrap().is_ok());
}
