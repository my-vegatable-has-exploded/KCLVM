use crate::langserver;
use crate::langserver::go_to_def::go_to_def;
use kclvm_error::Position;
use tower_lsp::lsp_types::Location;

use super::location_to_position;

/// Find all references of the item at the cursor location.
pub fn find_refs(path: String, pos: Position) -> Vec<Location> {
    let declaration = go_to_def(path.clone(), pos.clone());
    let search = {
        move |decl: Location| {
            let name = langserver::word_at_pos(pos);
            if name.is_none() {
                return vec![];
            }
            // Get identifiers with same name
            let candidates = langserver::match_word(path.clone(), name.unwrap());
            // Check if the definition of candidate and declartion are the same
            let refs: Vec<Location> = candidates
                .into_iter()
                .filter(|x| {
                    go_to_def(path.clone(), location_to_position(x.clone())).as_ref() == Some(&decl)
                })
                .collect();
            refs
        }
    };
    match declaration {
        Some(decl) => search(decl),
        None => Vec::new(),
    }
}
