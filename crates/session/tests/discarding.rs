//! Results are thrown away when the alignment they were built on changes (`Session::discard_results`).

mod common;

use common::{align_settings, point_settings, processed};
use planets_core::wavelet::{ColourMode, Region};
use planets_session::{
    ImageSource, LimitSettings, NoProgress, Session, SessionError, wavelet_default_settings,
};

/// Everything built on an alignment is gone: stack, sharpened image, preview source, colour source and the two
/// images *Save image* writes.
fn assert_discarded(session: &Session) {
    assert_eq!(
        session.stacked_image().unwrap_err(),
        SessionError::NotStacked
    );
    assert_eq!(
        session.require_image(ImageSource::Stack),
        Err(SessionError::NoStackedImage)
    );
    assert_eq!(
        session.require_image(ImageSource::Wavelet),
        Err(SessionError::NoProcessedImage)
    );
    let preview = session.wavelet_preview(None, Region::full(8, 8), ColourMode::Colour);
    assert_eq!(preview.err(), Some(SessionError::NoImageToSharpen));
    let adjust = session.color_histogram(&colors(), true);
    assert_eq!(adjust.err(), Some(SessionError::NoImageToAdjust));
}

/// Everything is in place.
fn assert_kept(session: &Session) {
    assert!(session.stacked_image().is_ok());
    assert!(session.require_image(ImageSource::Stack).is_ok());
    assert!(session.require_image(ImageSource::Wavelet).is_ok());
    assert!(
        session
            .wavelet_preview(None, Region::full(8, 8), ColourMode::Colour)
            .is_ok()
    );
    assert!(session.color_histogram(&colors(), true).is_ok());
}

fn colors() -> planets_session::ColorSettings {
    serde_json::from_value(serde_json::json!({
        "shifts": { "red": { "x": 0, "y": 0 }, "green": { "x": 0, "y": 0 }, "blue": { "x": 0, "y": 0 } },
        "red": 100, "green": 100, "blue": 100, "black": 0, "white": 255,
        "gamma": 1, "contrast": 100, "brightness": 0
    }))
    .unwrap()
}

#[test]
fn the_fixture_holds_every_result() {
    let (session, _) = processed("fixture");
    assert_kept(&session);
}

#[test]
fn new_alignpoints_discard_everything_built_on_the_old_alignment() {
    let (session, _) = processed("discard-points");
    session.set_alignpoints(0, &point_settings()).unwrap();
    assert_discarded(&session);
}

#[test]
fn aligning_again_discards_everything_even_when_it_is_cancelled() {
    let (session, _) = processed("discard-align");
    // Cancel arrives with the first frame, so the run never finishes.
    let cancel = |_, _, _, _| session.cancel(planets_session::Operation::Align);
    let result = session.align(&align_settings(), &cancel);
    assert_eq!(result.err(), Some(SessionError::Cancelled));
    assert_discarded(&session);
    // The cancelled run left no registration either: Limit has nothing to work on.
    let limit = session.limit(&LimitSettings::BestCount(3));
    assert_eq!(limit.err(), Some(SessionError::NotAligned));
}

#[test]
fn a_finished_align_discards_what_was_built_on_the_old_one() {
    let (session, _) = processed("discard-align-done");
    session.align(&align_settings(), &NoProgress).unwrap();
    assert_discarded(&session);
}

#[test]
fn limit_discards_the_stack_but_keeps_the_alignment() {
    let (session, _) = processed("discard-limit");
    let limited = session.limit(&LimitSettings::BestCount(3)).unwrap();
    assert_eq!(limited.frames.len(), 3);
    assert_discarded(&session);
    // The registration stays: Limit works again without Align.
    assert!(session.limit(&LimitSettings::BestCount(2)).is_ok());
}

#[test]
fn moving_the_threshold_slider_keeps_the_results() {
    let (session, _) = processed("threshold");
    let points = session.set_alignpoint_threshold(1).unwrap();
    assert_eq!(points.threshold, 1);
    assert_kept(&session);
}

#[test]
fn a_new_wavelet_run_does_not_touch_the_stack() {
    let (session, _) = processed("wavelet-again");
    session
        .wavelet_do_all(&wavelet_default_settings(), ColourMode::Colour, &NoProgress)
        .unwrap();
    assert_kept(&session);
}
