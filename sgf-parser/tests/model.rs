#[cfg(test)]
mod model_tests {
    use sgf_parser::*;

    #[test]
    fn can_get_unknown_nodes() {
        let tree: GameTree = parse("(;B[dc];W[ef]AC[23](;B[dd])(;AS[234]))").unwrap();
        let unknowns = tree.get_unknown_nodes();
        assert_eq!(unknowns.len(), 2);
        assert_eq!(
            *unknowns[0],
            GameNode {
                tokens: vec![
                    SgfToken::Move {
                        color: Color::White,
                        coordinate_or_pass: Some((5, 6))
                    },
                    SgfToken::Unknown(("AC".to_string(), "23".to_string()))
                ]
            }
        );
        assert_eq!(
            *unknowns[1],
            GameNode {
                tokens: vec![SgfToken::Unknown(("AS".to_string(), "234".to_string()))]
            }
        );
    }

    #[test]
    fn can_get_invalid_nodes() {
        let tree: GameTree = parse("(;B[dc];W[foobar](;B[dd])(;B[234]))").unwrap();
        let unknowns = tree.get_invalid_nodes();
        assert_eq!(unknowns.len(), 2);
        assert_eq!(
            *unknowns[0],
            GameNode {
                tokens: vec![SgfToken::Invalid(("W".to_string(), "foobar".to_string()))]
            }
        );
        assert_eq!(
            *unknowns[1],
            GameNode {
                tokens: vec![SgfToken::Invalid(("B".to_string(), "234".to_string()))]
            }
        );
    }

    #[test]
    fn can_iterate_over_simple_tree() {
        let tree: GameTree = parse("(;B[dc];W[ef])").unwrap();
        let mut iter = tree.iter();

        assert_eq!(
            iter.next(),
            Some(&GameNode {
                tokens: vec![SgfToken::Move {
                    color: Color::Black,
                    coordinate_or_pass: Some((4, 3))
                }]
            })
        );
        assert_eq!(
            iter.next(),
            Some(&GameNode {
                tokens: vec![SgfToken::Move {
                    color: Color::White,
                    coordinate_or_pass: Some((5, 6))
                }]
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn can_iterate_with_branch() {
        let tree: GameTree = parse("(;B[dc];W[ef](;B[aa])(;B[cc]))").unwrap();
        let mut iter = tree.iter();

        assert_eq!(
            iter.next(),
            Some(&GameNode {
                tokens: vec![SgfToken::Move {
                    color: Color::Black,
                    coordinate_or_pass: Some((4, 3))
                }]
            })
        );
        assert_eq!(
            iter.next(),
            Some(&GameNode {
                tokens: vec![SgfToken::Move {
                    color: Color::White,
                    coordinate_or_pass: Some((5, 6))
                }]
            })
        );
        assert_eq!(
            iter.next(),
            Some(&GameNode {
                tokens: vec![SgfToken::Move {
                    color: Color::Black,
                    coordinate_or_pass: Some((1, 1))
                }]
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iterator_can_switch_branch() {
        let tree: GameTree = parse("(;B[dc];W[ef](;B[aa])(;B[cc]))").unwrap();
        let mut iter = tree.iter();

        assert!(iter.has_variations());
        assert_eq!(iter.count_variations(), 2);

        assert!(iter.pick_variation(1).is_ok());

        assert_eq!(
            iter.next(),
            Some(&GameNode {
                tokens: vec![SgfToken::Move {
                    color: Color::Black,
                    coordinate_or_pass: Some((4, 3))
                }]
            })
        );
        assert_eq!(
            iter.next(),
            Some(&GameNode {
                tokens: vec![SgfToken::Move {
                    color: Color::White,
                    coordinate_or_pass: Some((5, 6))
                }]
            })
        );
        assert_eq!(
            iter.next(),
            Some(&GameNode {
                tokens: vec![SgfToken::Move {
                    color: Color::Black,
                    coordinate_or_pass: Some((3, 3))
                }]
            })
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn count_tree_length() {
        let tree: GameTree = parse("(;B[dc];W[ef](;B[aa])(;B[cc];W[dd]))").unwrap();
        assert_eq!(tree.count_max_nodes(), 4);
    }
}
