use kclvm_error::Position;
use kclvm_parser::load_program;
use kclvm_sema::resolver::{resolve_program, scope::ScopeObject};
use tower_lsp::lsp_types::{Location, Range, Url};

use super::word_at_pos;

/// Get the definition of an identifier.
pub fn go_to_def(path: String, pos: Position) -> Option<Location> {
    let mut program = load_program(&[&path], None).unwrap();
    let scope = resolve_program(&mut program);

    let name = word_at_pos(pos.clone());
    let mut obj_s: Option<ScopeObject> = None;
    if name.is_some() {
        let name = name.unwrap();
        for s in scope.scope_map.values() {
            let s = s.borrow_mut();
            match s.lookup(&name) {
                Some(obj) => {
                    obj_s = Some((*obj).borrow().clone());
                    break;
                }
                None => continue,
            }
        }
    };
    match obj_s {
        Some(obj) => {
            let start_position = tower_lsp::lsp_types::Position::new(
                obj.start.line as u32 - 1,
                obj.start.column.unwrap_or(0) as u32,
            );
            let end_position = tower_lsp::lsp_types::Position::new(
                obj.end.line as u32 - 1,
                obj.end.column.unwrap_or(0) as u32,
            );
            let range = Range::new(start_position, end_position);
            Some(Location::new(Url::from_file_path(path).unwrap(), range))
        }
        None => None,
    }
}

/// Get the definition of an identifier.
pub fn go_to_def_test(path: &str, pos: Position) -> Option<Position> {
    let mut program = load_program(&[path], None).unwrap();
    let scope = resolve_program(&mut program);
    // let a = scope.main_scope().unwrap().borrow_mut();
    let name = word_at_pos(pos.clone());
    println!("{:?}", name);
    if name.is_some() {
        let name = name.unwrap();
        for s in scope.scope_map.values() {
            let s = s.borrow_mut();
            match s.lookup(&name) {
                Some(obj) => println!("{:?}", obj),
                None => {}
            }
        }
    }

    Some(pos)
}
