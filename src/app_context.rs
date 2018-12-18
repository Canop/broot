use crate::verbs::VerbStore;

/// The immutable container that can be passed around to provide
/// the configuration things
pub struct AppContext {
    pub verb_store: VerbStore,
    pub output_path: Option<String>,
}
