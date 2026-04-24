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

    pub fn extend(&mut self, tracks: Vec<Arc<Track>>) -> Vec<Arc<Track>> {
        let paths: HashSet<&PathBuf> = self.tracks.iter().map(|track| &track.path).collect();
        let new_tracks: Vec<Arc<Track>> = tracks
            .into_iter()
            .filter(|track| !paths.contains(&track.path))
            .collect();
        self.tracks.extend(new_tracks.iter().cloned());
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
        let index = self.selected.as_ref().and_then(|selected| {
            self.matching
                .iter()
                .position(|track| Arc::ptr_eq(track, selected))
        });
        self.selected = match index {
            None => self.matching.first().cloned(),
            Some(index) => self
                .matching
                .get(index + 1)
                .cloned()
                .or_else(|| self.selected.clone()),
        };
    }

    pub fn select_previous(&mut self) {
        let index = self.selected.as_ref().and_then(|selected| {
            self.matching
                .iter()
                .position(|track| Arc::ptr_eq(track, selected))
        });
        self.selected = match index {
            None | Some(0) => self.matching.first().cloned(),
            Some(index) => self.matching.get(index - 1).cloned(),
        };
    }

    pub fn selected(&self) -> Option<&Arc<Track>> {
        self.selected.as_ref()
    }

    pub fn set_current_and_selected(&mut self, track: &Arc<Track>) {
        self.current = Some(track.clone());
        self.selected = Some(track.clone());
    }
}

#[cfg(test)]
#[path = "list_test.rs"]
mod tests;

#[derive(Default)]
pub struct List {
    current: Option<Arc<Track>>,
    matching: Vec<Arc<Track>>,
    search_query: String,
    selected: Option<Arc<Track>>,
    tracks: Vec<Arc<Track>>,
}
