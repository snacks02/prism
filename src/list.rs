use {
    crate::track::Track,
    nucleo::{
        Utf32String,
        pattern::{
            CaseMatching,
            Normalization,
            Pattern,
        },
    },
    std::{
        cmp::Reverse,
        collections::HashSet,
        path::PathBuf,
        sync::Arc,
    },
};

impl List {
    fn refresh_matching(&mut self) {
        let pattern = Pattern::parse(
            &self.search_query,
            CaseMatching::Ignore,
            Normalization::Smart,
        );
        let mut matcher = Default::default();
        let mut scored: Vec<(Arc<Track>, u32)> = self
            .tracks
            .iter()
            .filter_map(|track| {
                pattern
                    .score(
                        Utf32String::from(format!(
                            "{} {} {}",
                            track.album_str(),
                            track.artist_str(),
                            track.title_str()
                        ))
                        .slice(..),
                        &mut matcher,
                    )
                    .map(|score| (track.clone(), score))
            })
            .collect();
        scored.sort_unstable_by_key(|&(_, score)| Reverse(score));
        self.matching = scored.into_iter().map(|(track, _)| track).collect();
    }

    pub fn current(&self) -> Option<&Arc<Track>> {
        self.current.as_ref()
    }

    pub fn extend(&mut self, tracks: Vec<Track>) -> Vec<Arc<Track>> {
        let paths: HashSet<&PathBuf> = self.tracks.iter().map(|track| &track.path).collect();
        let new_tracks: Vec<Arc<Track>> = tracks
            .into_iter()
            .filter(|track| !paths.contains(&track.path))
            .map(Arc::new)
            .collect();
        self.tracks.extend_from_slice(&new_tracks);
        self.refresh_matching();
        new_tracks
    }

    pub fn matching(&self) -> &[Arc<Track>] {
        &self.matching
    }

    pub fn search(&mut self, query: String) {
        self.search_query = query;
        self.refresh_matching();
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn select_next(&mut self) {
        let Some(selected) = self.selected.as_ref() else {
            self.selected = self.matching.first().cloned();
            return;
        };
        self.selected = self
            .matching
            .iter()
            .skip_while(|&track| !Arc::ptr_eq(selected, track))
            .nth(1)
            .cloned()
            .or_else(|| self.selected.clone());
    }

    pub fn select_previous(&mut self) {
        let Some(selected) = self.selected.as_ref() else {
            self.selected = self.matching.first().cloned();
            return;
        };
        self.selected = self
            .matching
            .iter()
            .take_while(|&track| !Arc::ptr_eq(selected, track))
            .last()
            .cloned()
            .or_else(|| self.selected.clone());
    }

    pub fn selected(&self) -> Option<&Arc<Track>> {
        self.selected.as_ref()
    }

    pub fn set_current_and_selected(&mut self, track: &Arc<Track>) {
        self.current = Some(track.clone());
        self.selected = Some(track.clone());
    }

    pub fn tracks(&self) -> &[Arc<Track>] {
        &self.tracks
    }
}

#[cfg(test)]
#[path = "list_test.rs"]
mod tests;

#[derive(Debug, Default)]
pub struct List {
    current: Option<Arc<Track>>,
    matching: Vec<Arc<Track>>,
    search_query: String,
    selected: Option<Arc<Track>>,
    tracks: Vec<Arc<Track>>,
}
