#[cfg(test)]
mod node_tests {
    use sgf_parser::*;

    #[test]
    fn can_convert_node_to_string() {
        let node = GameNode {
            tokens: vec![
                SgfToken::PlayerName {
                    color: Color::Black,
                    name: "black".to_string(),
                },
                SgfToken::PlayerName {
                    color: Color::White,
                    name: "white".to_string(),
                },
            ],
        };
        let string_node: String = node.into();
        assert_eq!(string_node, ";PB[black]PW[white]");
    }
}
