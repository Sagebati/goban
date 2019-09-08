#[cfg(test)]
mod parser_tests {
    use sgf_parser::*;

    #[test]
    fn errors_on_invalid_root_token_placement() {
        let sgf = parse("(;KM[6.5];SZ[19])");
        match sgf {
            Err(ref e) => assert_eq!(e.kind, SgfErrorKind::InvalidRootTokenPlacement),
            _ => assert!(false),
        }
    }

    #[test]
    fn can_parse_komi() {
        let sgf = parse("(;KM[6.5])");
        assert!(sgf.is_ok());
        let sgf = sgf.unwrap();
        assert_eq!(
            sgf,
            GameTree {
                nodes: vec![GameNode {
                    tokens: vec![SgfToken::Komi(6.5f32)]
                }],
                variations: vec![]
            }
        );
    }

    #[test]
    fn can_ignore_lowercase_characters() {
        let sgf = parse("(;CopyRight[2017])");
        assert!(sgf.is_ok());
        let sgf = sgf.unwrap();
        assert_eq!(
            sgf,
            GameTree {
                nodes: vec![GameNode {
                    tokens: vec![SgfToken::Copyright("2017".to_string())],
                }],
                variations: vec![]
            }
        );
    }

    #[test]
    fn can_parse_game_tree_single_node() {
        let sgf = parse("(;B[dc]BL[3498])");
        assert!(sgf.is_ok());
        let sgf = sgf.unwrap();
        assert_eq!(
            sgf,
            GameTree {
                nodes: vec![GameNode {
                    tokens: vec![
                        SgfToken::Move {
                            color: Color::Black,
                            coordinate_or_pass: Some((4, 3))
                        },
                        SgfToken::Time {
                            color: Color::Black,
                            time: 3498
                        }
                    ],
                }],
                variations: vec![]
            }
        );
    }

    #[test]
    fn can_parse_game_tree_two_nodes() {
        let sgf = parse("(;B[dc];W[ef])");
        assert!(sgf.is_ok());
        let sgf = sgf.unwrap();
        assert_eq!(
            sgf,
            GameTree {
                nodes: vec![
                    GameNode {
                        tokens: vec![SgfToken::Move {
                            color: Color::Black,
                            coordinate_or_pass: Some((4, 3))
                        }],
                    },
                    GameNode {
                        tokens: vec![SgfToken::Move {
                            color: Color::White,
                            coordinate_or_pass: Some((5, 6))
                        }],
                    }
                ],
                variations: vec![]
            }
        );
    }

    #[test]
    fn can_parse_game_tree_simple_branch() {
        let sgf = parse("(;B[aa](;W[bb])(;W[cc]))");
        assert!(sgf.is_ok());
        let sgf = sgf.unwrap();
        assert_eq!(
            sgf,
            GameTree {
                nodes: vec![GameNode {
                    tokens: vec![SgfToken::Move {
                        color: Color::Black,
                        coordinate_or_pass: Some((1, 1))
                    }],
                },],
                variations: vec![
                    GameTree {
                        nodes: vec![GameNode {
                            tokens: vec![SgfToken::Move {
                                color: Color::White,
                                coordinate_or_pass: Some((2, 2))
                            }],
                        },],
                        variations: vec![]
                    },
                    GameTree {
                        nodes: vec![GameNode {
                            tokens: vec![SgfToken::Move {
                                color: Color::White,
                                coordinate_or_pass: Some((3, 3))
                            }],
                        },],
                        variations: vec![]
                    }
                ]
            }
        );
    }

    #[test]
    fn can_parse_game_information() {
        let sgf = parse("(;EV[event]PB[black]PW[white]C[comment];B[aa])");
        assert!(sgf.is_ok());
        let sgf = sgf.unwrap();
        assert_eq!(
            sgf,
            GameTree {
                nodes: vec![
                    GameNode {
                        tokens: vec![
                            SgfToken::Event("event".to_string()),
                            SgfToken::PlayerName {
                                color: Color::Black,
                                name: "black".to_string()
                            },
                            SgfToken::PlayerName {
                                color: Color::White,
                                name: "white".to_string()
                            },
                            SgfToken::Comment("comment".to_string()),
                        ],
                    },
                    GameNode {
                        tokens: vec![SgfToken::Move {
                            color: Color::Black,
                            coordinate_or_pass: Some((1, 1))
                        }],
                    }
                ],
                variations: vec![]
            }
        );
    }

    #[test]
    fn can_parse_unkown_tags() {
        let sgf = parse("(;B[dc];FO[asdf];W[ef])");
        assert!(sgf.is_ok());
        let sgf = sgf.unwrap();
        assert_eq!(
            sgf,
            GameTree {
                nodes: vec![
                    GameNode {
                        tokens: vec![SgfToken::Move {
                            color: Color::Black,
                            coordinate_or_pass: Some((4, 3))
                        }],
                    },
                    GameNode {
                        tokens: vec![SgfToken::Unknown(("FO".to_string(), "asdf".to_string())),],
                    },
                    GameNode {
                        tokens: vec![SgfToken::Move {
                            color: Color::White,
                            coordinate_or_pass: Some((5, 6))
                        }],
                    }
                ],
                variations: vec![]
            }
        );
    }

    #[test]
    fn can_parse_wrapped_comment() {
        let sgf = parse("(;C[a [wrapped\\] comment])");
        assert!(sgf.is_ok());
        let sgf = sgf.unwrap();
        assert_eq!(
            sgf,
            GameTree {
                nodes: vec![GameNode {
                    tokens: vec![SgfToken::Comment("a [wrapped\\] comment".to_string()),],
                },],
                variations: vec![]
            }
        );
    }
}
