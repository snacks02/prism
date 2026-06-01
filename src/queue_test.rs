use {
    super::*,
    crate::track::Track,
    std::sync::Arc,
};

fn new_track(title: &str) -> Arc<Track> {
    Arc::new(Track {
        title: Some(title.to_string()),
        ..Default::default()
    })
}

mod extend {
    use super::*;

    #[test]
    fn adds_the_tracks() {
        let mut queue = Queue::default();
        let track = new_track("track");
        queue.extend(vec![track.clone()]);

        assert_eq!(&queue.tracks, &[track]);
    }

    #[test]
    fn shuffles_the_tracks_when_shuffle_is_enabled() {
        fastrand::seed(2);
        let mut queue = Queue::default();
        queue.shuffle = true;
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");
        queue.extend(vec![track_1.clone(), track_2.clone()]);

        assert_eq!(queue.tracks, &[track_2, track_1]);
    }
}

mod next {
    use super::*;

    #[test]
    fn returns_none_when_at_the_last_track() {
        let mut queue = Queue::default();
        let track = new_track("track");
        queue.tracks = vec![track.clone()];

        assert!(queue.next(Some(&track)).is_none());
    }

    #[test]
    fn returns_the_first_track_when_at_the_last_track_and_repeat_is_enabled() {
        let mut queue = Queue::default();
        let track = new_track("track");
        queue.repeat = true;
        queue.tracks = vec![track.clone()];

        assert!(Arc::ptr_eq(&queue.next(Some(&track)).unwrap(), &track));
    }

    #[test]
    fn returns_the_first_track_when_none_is_current() {
        let mut queue = Queue::default();
        let track = new_track("track");
        queue.tracks = vec![track.clone()];

        assert!(Arc::ptr_eq(&queue.next(None).unwrap(), &track));
    }

    #[test]
    fn returns_the_next_track() {
        let mut queue = Queue::default();
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");
        queue.tracks = vec![track_1.clone(), track_2.clone()];

        assert!(Arc::ptr_eq(&queue.next(Some(&track_1)).unwrap(), &track_2));
    }
}

mod previous {
    use super::*;

    #[test]
    fn returns_none_when_at_the_first_track() {
        let mut queue = Queue::default();
        let track = new_track("track");
        queue.tracks = vec![track.clone()];

        assert!(queue.previous(Some(&track)).is_none());
    }

    #[test]
    fn returns_the_first_track_when_none_is_current() {
        let mut queue = Queue::default();
        let track = new_track("track");
        queue.tracks = vec![track.clone()];

        assert!(Arc::ptr_eq(&queue.previous(None).unwrap(), &track));
    }

    #[test]
    fn returns_the_last_track_when_at_the_first_track_and_repeat_is_enabled() {
        let mut queue = Queue::default();
        let track = new_track("track");
        queue.repeat = true;
        queue.tracks = vec![track.clone()];

        assert!(Arc::ptr_eq(&queue.previous(Some(&track)).unwrap(), &track));
    }

    #[test]
    fn returns_the_previous_track() {
        let mut queue = Queue::default();
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");
        queue.tracks = vec![track_1.clone(), track_2.clone()];

        assert!(Arc::ptr_eq(
            &queue.previous(Some(&track_2)).unwrap(),
            &track_1
        ));
    }
}

mod repeat {
    use super::*;

    #[test]
    fn returns_the_repeat_value() {
        assert!(!Queue::default().repeat());
    }
}

mod repeat_disable {
    use super::*;

    #[test]
    fn disables_repeat() {
        let mut queue = Queue::default();
        queue.repeat = true;
        queue.repeat_disable();

        assert!(!queue.repeat);
    }
}

mod repeat_enable {
    use super::*;

    #[test]
    fn enables_repeat() {
        let mut queue = Queue::default();
        queue.repeat_enable();

        assert!(queue.repeat);
    }
}

mod shuffle {
    use super::*;

    #[test]
    fn returns_the_shuffle_value() {
        assert!(!Queue::default().shuffle());
    }
}

mod shuffle_disable {
    use super::*;

    #[test]
    fn disables_shuffle_and_updates_the_tracks() {
        fastrand::seed(2);
        let mut queue = Queue::default();
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");
        let tracks = vec![track_1.clone(), track_2.clone()];
        queue.tracks = vec![track_1.clone(), track_2.clone()];
        queue.shuffle_enable();
        queue.shuffle_disable(&tracks);

        assert!(!queue.shuffle);
        assert_eq!(queue.tracks, tracks);
    }
}

mod shuffle_enable {
    use super::*;

    #[test]
    fn enables_shuffle_and_shuffles_the_tracks() {
        fastrand::seed(2);
        let mut queue = Queue::default();
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");
        queue.tracks = vec![track_1.clone(), track_2.clone()];
        queue.shuffle_enable();

        assert!(queue.shuffle);
        assert_eq!(queue.tracks, &[track_2, track_1]);
    }
}
