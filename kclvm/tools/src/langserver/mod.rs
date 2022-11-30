#![allow(dead_code)]

use std::fmt::Debug;

use crate::util;
use anyhow::Result;
use kclvm_error::Position;
use tower_lsp::lsp_types::*;
use tower_lsp::lsp_types::{Location, Range};

pub mod find_refs;
pub mod go_to_def;
pub mod word_map;

#[cfg(test)]
mod tests;

// LineWord describes an arbitrary word in a certain line including
// start position, end position and the word itself.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LineWord {
    startpos: u64,
    endpos: u64,
    word: String,
}

// Get the word of the position.
pub fn word_at_pos(pos: Position) -> Option<String> {
    let text = read_file(&pos.filename);
    if text.is_err() {
        return None;
    }
    let text = text.unwrap();
    let lines: Vec<&str> = text.lines().collect();
    if pos.line >= lines.len() as u64 {
        return None;
    }
    pos.column?;
    let col = pos.column.unwrap();
    let line_words = line_to_words(lines[pos.line as usize].to_string());
    if line_words.is_empty()
        || col < line_words.first().unwrap().startpos
        || col >= line_words.last().unwrap().endpos
    {
        return None;
    }
    for line_word in line_words.into_iter() {
        if col >= line_word.startpos && col < line_word.endpos {
            return Some(line_word.word);
        }
    }
    None
}

pub fn read_file(path: &String) -> Result<String> {
    let text = std::fs::read_to_string(&path)?;
    Ok(text)
}

// Split one line into identifier words.
pub fn line_to_words(text: String) -> Vec<LineWord> {
    let mut chars: Vec<char> = text.chars().collect();
    chars.push('\n');
    let mut start_pos = usize::MAX;
    let mut continue_pos = usize::MAX - 1; // avoid overflow
    let mut prev_word = false;
    let mut words: Vec<LineWord> = vec![];
    for (i, ch) in chars.iter().enumerate() {
        let is_id_start = rustc_lexer::is_id_start(*ch);
        let is_id_continue = rustc_lexer::is_id_continue(*ch);
        // If the character is valid identfier start and the previous character is not valid identifier continue, mark the start position.
        if is_id_start && !prev_word {
            start_pos = i;
        }
        match is_id_continue {
            true => {
                // Continue searching for the end position.
                if start_pos != usize::MAX {
                    continue_pos = i;
                }
            }
            false => {
                // Find out the end position.
                if continue_pos + 1 == i {
                    words.push(LineWord {
                        startpos: start_pos as u64,
                        endpos: i as u64,
                        word: chars[start_pos..i].iter().collect::<String>().clone(),
                    });
                }
                // Reset the start position.
                start_pos = usize::MAX;
            }
        }
        prev_word = is_id_continue;
    }
    words
}

// Get all occurrences of the word in the entire path.
pub fn match_word(path: String, name: String) -> Vec<Location> {
    let mut res = vec![];
    let files = util::get_kcl_files(path, true);
    match files {
        Ok(files) => {
            // Searching in all files.
            for file in files.into_iter() {
                let text = read_file(&file);
                if text.is_err() {
                    continue;
                }
                let text = text.unwrap();
                let lines: Vec<&str> = text.lines().collect();
                for (li, line) in lines.into_iter().enumerate() {
                    // Get the matching results for each line.
                    let matched: Vec<Location> = line_to_words(line.to_string())
                        .into_iter()
                        .filter(|x| x.word == name)
                        .map(|x| lineword_to_location(file.clone(), li as u32, x))
                        .collect();
                    res.extend(matched);
                }
            }
        }
        Err(_) => {}
    }
    res
}

fn lineword_to_location(filename: String, line: u32, word: LineWord) -> Location {
    Location {
        uri: Url::from_file_path(filename).unwrap(),
        range: Range {
            start: tower_lsp::lsp_types::Position {
                line: line as u32,
                character: word.startpos as u32,
            },
            end: tower_lsp::lsp_types::Position {
                line: line as u32,
                character: word.endpos as u32, // TODO: check if it is correct, may need to -1
            },
        },
    }
}

fn position_to_location(pos: Position) -> Location {
    Location {
        uri: Url::from_file_path(pos.filename).unwrap(),
        range: Range {
            start: tower_lsp::lsp_types::Position {
                line: pos.line as u32,
                character: pos.column.unwrap() as u32,
            },
            end: tower_lsp::lsp_types::Position {
                line: pos.line as u32,
                character: pos.column.unwrap() as u32,
            },
        },
    }
}

fn location_to_position(loc: Location) -> Position {
    Position {
        filename: loc.uri.path().to_string(),
        line: loc.range.start.line as u64,
        column: Some(loc.range.start.character as u64),
    }
}

// Convert pos format
// The position in lsp protocol is different with position in ast node whose line number is 1 based.
pub fn kcl_pos_to_lsp_pos(pos: Position) -> Position {
    Position {
        filename: pos.filename,
        line: pos.line - 1,
        column: pos.column,
    }
}

pub fn lsp_pos_to_kcl_pos(pos: Position) -> Position {
    Position {
        filename: pos.filename,
        line: pos.line + 1,
        column: pos.column,
    }
}
