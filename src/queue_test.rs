use super::*;

#[test]
fn set_clones_with_shuffle() {
    let mut queue = Queue::default();
    queue.toggle_shuffle();
    let tracks = helper::new_tracks();
    queue.set(&tracks[0], tracks.clone());
    assert!(!Arc::ptr_eq(&queue.tracks, &tracks));
}

#[test]
fn set_does_not_clone_without_shuffle() {
    let mut queue = Queue::default();
    let tracks = helper::new_tracks();
    queue.set(&tracks[0], tracks.clone());
    assert!(Arc::ptr_eq(&queue.tracks, &tracks));
}

mod helper {
    use {
        super::*,
        std::path::PathBuf,
    };

    pub fn new_tracks() -> Arc<Vec<Arc<Track>>> {
        Arc::new(vec![Arc::new(Track {
            album: None,
            artist: None,
            duration: None,
            path: PathBuf::new(),
            replay_gain: None,
            title: None,
        })])
    }
}
