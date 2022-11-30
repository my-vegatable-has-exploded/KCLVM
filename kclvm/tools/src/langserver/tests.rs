use crate::langserver;
use crate::langserver::LineWord;
use kclvm_error::Position as KCLPOS;

#[cfg(test)]
mod tests {
    use crate::langserver::{
        find_refs::find_refs,
        go_to_def::{go_to_def, go_to_def_test},
    };
    use std::env;
    use tower_lsp::lsp_types::{Location, Position, Range, Url};

    use super::*;
    use std::fs;

    fn check_line_to_words(code: &str, expect: Vec<LineWord>) {
        assert_eq!(langserver::line_to_words(code.to_string()), expect);
    }

    trait UnorderedEq {
        fn unordered_eq(&self, other: &Self) -> bool;
    }

    impl<T: Eq + Clone> UnorderedEq for Vec<T> {
        fn unordered_eq(&self, other: &Self) -> bool {
            if self.len() != other.len() {
                return false;
            }

            let mut match_count = 0;

            let mut other_to_check: Self = Vec::new();
            other_to_check.clone_from(other);

            for item in self {
                let index_in_other = other_to_check.iter().position(|e| e == item);

                if let Some(index) = index_in_other {
                    other_to_check.remove(index);
                    match_count += 1;
                }
            }

            println!("{}", match_count);
            self.len() == match_count
        }
    }

    #[test]
    fn test_line_to_words() {
        let datas = vec![
            "alice_first_name = \"alice\"",
            "0lice_first_name = \"alic0\"",
            "alice = p.Parent { name: \"alice\" }",
        ];
        let expect = vec![
            vec![
                LineWord {
                    startpos: 0,
                    endpos: 16,
                    word: "alice_first_name".to_string(),
                },
                LineWord {
                    startpos: 20,
                    endpos: 25,
                    word: "alice".to_string(),
                },
            ],
            vec![LineWord {
                startpos: 20,
                endpos: 25,
                word: "alic0".to_string(),
            }],
            vec![
                LineWord {
                    startpos: 0,
                    endpos: 5,
                    word: "alice".to_string(),
                },
                LineWord {
                    startpos: 8,
                    endpos: 9,
                    word: "p".to_string(),
                },
                LineWord {
                    startpos: 10,
                    endpos: 16,
                    word: "Parent".to_string(),
                },
                LineWord {
                    startpos: 19,
                    endpos: 23,
                    word: "name".to_string(),
                },
                LineWord {
                    startpos: 26,
                    endpos: 31,
                    word: "alice".to_string(),
                },
            ],
        ];
        for i in 0..datas.len() {
            check_line_to_words(datas[i], expect[i].clone());
        }
    }

    #[test]
    fn test_word_at_pos() {
        let path_prefix = "./src/langserver/".to_string();
        let datas = vec![
            KCLPOS {
                filename: (path_prefix.clone() + "test_data/inherit.k"),
                line: 0,
                column: Some(0),
            },
            KCLPOS {
                filename: (path_prefix.clone() + "test_data/inherit.k"),
                line: 1,
                column: Some(5),
            },
            KCLPOS {
                filename: (path_prefix.clone() + "test_data/inherit.k"),
                line: 3,
                column: Some(7),
            },
            KCLPOS {
                filename: (path_prefix.clone() + "test_data/inherit.k"),
                line: 3,
                column: Some(10),
            },
            KCLPOS {
                filename: (path_prefix.clone() + "test_data/inherit.k"),
                line: 4,
                column: Some(8),
            },
            KCLPOS {
                filename: (path_prefix + "test_data/inherit.k"),
                line: 4,
                column: Some(100),
            },
        ];
        let expect = vec![
            Some("schema".to_string()),
            Some("name".to_string()),
            Some("Son".to_string()),
            None,
            None,
            None,
        ];
        for i in 0..datas.len() {
            assert_eq!(langserver::word_at_pos(datas[i].clone()), expect[i]);
        }
    }

    #[test]
    fn test_match_word() {
        let parent_path: String = env::current_dir().unwrap().to_str().unwrap().into();
        // println!("The current directory is {}", parent_path.display());
        let path = parent_path + "/src/langserver/test_data/test_word_workspace";
        let datas = vec![String::from("Son")];
        let except = vec![vec![
            Location {
                uri: Url::from_file_path(path.clone() + "/inherit_pkg.k").unwrap(),
                range: Range {
                    start: Position {
                        line: 2,
                        character: 7,
                    },
                    end: Position {
                        line: 2,
                        character: 10,
                    },
                },
            },
            Location {
                uri: Url::from_file_path(path.clone() + "/inherit.k").unwrap(),
                range: Range {
                    start: Position {
                        line: 3,
                        character: 7,
                    },
                    end: Position {
                        line: 3,
                        character: 10,
                    },
                },
            },
            Location {
                uri: Url::from_file_path(path.clone() + "/inherit.k").unwrap(),
                range: Range {
                    start: Position {
                        line: 7,
                        character: 16,
                    },
                    end: Position {
                        line: 7,
                        character: 19,
                    },
                },
            },
        ]];
        for i in 0..datas.len() {
            assert!(langserver::match_word(path.clone(), datas[i].clone()).unordered_eq(&except[i]));
        }
    }

    #[test]
    fn test_word_map() {
        let parent_path: String = env::current_dir().unwrap().to_str().unwrap().into();
        let path = parent_path + "/src/langserver/test_data/test_word_workspace_map";
        let mut mp = langserver::word_map::WorkSpaceWordMap::new(path.clone());
        mp.build();
        let _res = fs::rename(
            path.clone() + "/inherit_pkg.k",
            path.clone() + "/inherit_bak.k",
        );
        mp.rename_file(
            path.clone() + "/inherit_pkg.k",
            path.clone() + "/inherit_bak.k",
        );
        mp.delete_file(path.clone() + "/inherit.k");
        let _res = fs::rename(
            path.clone() + "/inherit_bak.k",
            path.clone() + "/inherit_pkg.k",
        );

        let except = vec![Location {
            uri: Url::from_file_path(path.clone() + "/inherit_bak.k").unwrap(),
            range: Range {
                start: Position {
                    line: 2,
                    character: 7,
                },
                end: Position {
                    line: 2,
                    character: 10,
                },
            },
        }];
        assert_eq!(mp.get(&String::from("Son")), Some(except));
    }

    // #[test]
    // fn test_word_map_aa() {
    //     let path = "/Users/zz/code/KCL-Models/fib.k";
    //     go_to_def_test(
    //         path,
    //         KCLPOS {
    //             filename: String::from("/Users/zz/code/KCL-Models/fib.k"),
    //             line: 14,
    //             column: Some(9),
    //         },
    //     );
    // }

    #[test]
    fn test_go_to_def() {
        let parent_path: String = env::current_dir().unwrap().to_str().unwrap().into();
        let path_prefix = parent_path + "/src/langserver/";
        let datas = vec![KCLPOS {
            filename: (path_prefix.clone() + "test_data/simple.k"),
            line: 1,
            column: Some(4),
        }];
        let expect = vec![Some(Location {
            uri: Url::from_file_path(path_prefix.clone() + "test_data/simple.k").unwrap(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 1,
                },
            },
        })];
        for i in 0..datas.len() {
            assert_eq!(
                go_to_def(datas[i].filename.clone(), datas[i].clone()),
                expect[i]
            );
        }
    }

    #[test]
    fn test_find_refs() {
        let parent_path: String = env::current_dir().unwrap().to_str().unwrap().into();
        let path_prefix = parent_path + "/src/langserver/";
        let datas = vec![KCLPOS {
            filename: (path_prefix.clone() + "test_data/simple.k"),
            line: 0,
            column: Some(0),
        }];
        let expect = vec![vec![
            Location {
                uri: Url::from_file_path(path_prefix.clone() + "test_data/simple.k").unwrap(),
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 1,
                    },
                },
            },
            Location {
                uri: Url::from_file_path(path_prefix.clone() + "test_data/simple.k").unwrap(),
                range: Range {
                    start: Position {
                        line: 1,
                        character: 4,
                    },
                    end: Position {
                        line: 1,
                        character: 5,
                    },
                },
            },
        ]];
        for i in 0..datas.len() {
            assert!(find_refs(datas[i].filename.clone(), datas[i].clone()).unordered_eq(&expect[i]));
        }
    }
}
