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

mod current {
    use super::*;

    #[test]
    fn returns_the_current_track() {
        let mut list = List::default();
        let track = new_track("track");
        list.current = Some(track.clone());

        assert!(Arc::ptr_eq(list.current().unwrap(), &track));
    }
}

mod extend {
    use super::*;

    #[test]
    fn adds_and_deduplicates_tracks() {
        let mut list = List::default();
        let track = new_track("track");

        assert_eq!(list.extend(vec![track.clone()]), &[track.clone()]);
        assert_eq!(list.matching, &[track.clone()]);

        assert_eq!(list.extend(vec![track.clone()]), &[]);
        assert_eq!(list.matching, &[track.clone()]);
    }

    #[test]
    fn updates_matching() {
        let mut list = List::default();
        list.search_query = "track_1".into();
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");

        list.extend(vec![track_1.clone(), track_2]);

        assert_eq!(list.matching, &[track_1]);
    }
}

mod matching {
    use super::*;

    #[test]
    fn returns_matching_tracks() {
        let mut list = List::default();
        let track = new_track("track");
        list.matching = vec![track.clone()];

        assert_eq!(list.matching(), &[track]);
    }
}

mod search {
    use super::*;

    #[test]
    fn updates_matching_and_search_query() {
        let mut list = List::default();
        let track_1 = new_track("track_1");
        list.tracks = vec![track_1.clone(), new_track("track_2")];
        list.search("track_1".into());

        assert_eq!(list.matching, &[track_1]);
        assert_eq!(list.search_query, "track_1");
    }
}

mod search_query {
    use super::*;

    #[test]
    fn returns_the_search_query() {
        let mut list = List::default();
        list.search_query = "search_query".into();

        assert_eq!(list.search_query(), "search_query");
    }
}

mod select_next {
    use super::*;

    #[test]
    fn selects_the_first_track_when_none_is_selected() {
        let mut list = List::default();
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");
        list.matching = vec![track_1.clone(), track_2];
        list.select_next();

        assert!(Arc::ptr_eq(list.selected.as_ref().unwrap(), &track_1));
    }

    #[test]
    fn selects_the_last_track_when_already_at_last() {
        let mut list = List::default();
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");
        list.matching = vec![track_1, track_2.clone()];
        list.selected = Some(track_2.clone());
        list.select_next();

        assert!(Arc::ptr_eq(list.selected.as_ref().unwrap(), &track_2));
    }

    #[test]
    fn selects_the_next_track() {
        let mut list = List::default();
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");
        list.matching = vec![track_1.clone(), track_2.clone()];
        list.selected = Some(track_1);
        list.select_next();

        assert!(Arc::ptr_eq(list.selected.as_ref().unwrap(), &track_2));
    }
}

mod select_previous {
    use super::*;

    #[test]
    fn selects_the_first_track_when_already_at_first() {
        let mut list = List::default();
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");
        list.matching = vec![track_1.clone(), track_2];
        list.selected = Some(track_1.clone());
        list.select_previous();

        assert!(Arc::ptr_eq(list.selected.as_ref().unwrap(), &track_1));
    }

    #[test]
    fn selects_the_first_track_when_none_is_selected() {
        let mut list = List::default();
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");
        list.matching = vec![track_1.clone(), track_2];
        list.select_previous();

        assert!(Arc::ptr_eq(list.selected.as_ref().unwrap(), &track_1));
    }

    #[test]
    fn selects_the_previous_track() {
        let mut list = List::default();
        let track_1 = new_track("track_1");
        let track_2 = new_track("track_2");
        list.matching = vec![track_1.clone(), track_2.clone()];
        list.selected = Some(track_2);
        list.select_previous();

        assert!(Arc::ptr_eq(list.selected.as_ref().unwrap(), &track_1));
    }
}

mod selected {
    use super::*;

    #[test]
    fn returns_the_selected_track() {
        let mut list = List::default();
        let track = new_track("track");
        list.selected = Some(track.clone());

        assert!(Arc::ptr_eq(list.selected().unwrap(), &track));
    }
}

mod set_current_and_selected {
    use super::*;

    #[test]
    fn updates_the_current_and_the_selected_tracks() {
        let mut list = List::default();
        let track = new_track("track");
        list.set_current_and_selected(&track);

        assert!(Arc::ptr_eq(list.current.as_ref().unwrap(), &track));
        assert!(Arc::ptr_eq(list.selected.as_ref().unwrap(), &track));
    }
}
