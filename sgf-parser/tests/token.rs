#[cfg(test)]
mod token_tests {
    use sgf_parser::*;

    #[test]
    fn can_parse_move_tokens() {
        let token = SgfToken::from_pair("B", "aa");
        assert_eq!(
            token,
            SgfToken::Move {
                color: Color::Black,
                coordinate_or_pass: Some((1, 1)),
            }
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "B[aa]");

        let token = SgfToken::from_pair("W", "kk");
        assert_eq!(
            token,
            SgfToken::Move {
                color: Color::White,
                coordinate_or_pass: Some((11, 11)),
            }
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "W[kk]");
    }

    #[test]
    fn can_parse_time_tokens() {
        let token = SgfToken::from_pair("BL", "1234");
        assert_eq!(
            token,
            SgfToken::Time {
                color: Color::Black,
                time: 1234,
            }
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "BL[1234]");

        let token = SgfToken::from_pair("WL", "34");
        assert_eq!(
            token,
            SgfToken::Time {
                color: Color::White,
                time: 34,
            }
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "WL[34]");
    }

    #[test]
    fn can_parse_name_tokens() {
        let token = SgfToken::from_pair("PB", "Honinbo Shusai");
        assert_eq!(
            token,
            SgfToken::PlayerName {
                color: Color::Black,
                name: "Honinbo Shusai".to_string(),
            }
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "PB[Honinbo Shusai]");

        let token = SgfToken::from_pair("PW", "Cho Chikun");
        assert_eq!(
            token,
            SgfToken::PlayerName {
                color: Color::White,
                name: "Cho Chikun".to_string(),
            }
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "PW[Cho Chikun]");
    }

    #[test]
    fn can_parse_rank_tokens() {
        let token = SgfToken::from_pair("BR", "3p");
        assert_eq!(
            token,
            SgfToken::PlayerRank {
                color: Color::Black,
                rank: "3p".to_string(),
            }
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "BR[3p]");

        let token = SgfToken::from_pair("WR", "5 kyu");
        assert_eq!(
            token,
            SgfToken::PlayerRank {
                color: Color::White,
                rank: "5 kyu".to_string(),
            }
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "WR[5 kyu]");
    }

    #[test]
    fn can_parse_komi_tokens() {
        let token = SgfToken::from_pair("KM", "4.5");
        assert_eq!(token, SgfToken::Komi(4.5));
        let string_token: String = token.into();
        assert_eq!(string_token, "KM[4.5]");
    }

    #[test]
    fn can_parse_size_tokens() {
        let token = SgfToken::from_pair("SZ", "19");
        assert_eq!(token, SgfToken::Size(19, 19));
        let string_token: String = token.into();
        assert_eq!(string_token, "SZ[19]");
    }

    #[test]
    fn can_parse_size_token_with_two_values() {
        let token = SgfToken::from_pair("SZ", "15:17");
        assert_eq!(token, SgfToken::Size(15, 17));
        let string_token: String = token.into();
        assert_eq!(string_token, "SZ[15:17]");
    }

    #[test]
    fn can_parse_time_limit_tokens() {
        let token = SgfToken::from_pair("TM", "1234");
        assert_eq!(token, SgfToken::TimeLimit(1234));
        let string_token: String = token.into();
        assert_eq!(string_token, "TM[1234]");
    }

    #[test]
    fn can_parse_event_tokens() {
        let token = SgfToken::from_pair("EV", "event");
        assert_eq!(token, SgfToken::Event("event".to_string()));
        let string_token: String = token.into();
        assert_eq!(string_token, "EV[event]");
    }

    #[test]
    fn can_parse_comment_tokens() {
        let token = SgfToken::from_pair("C", "comment");
        assert_eq!(token, SgfToken::Comment("comment".to_string()));
        let string_token: String = token.into();
        assert_eq!(string_token, "C[comment]");
    }

    #[test]
    fn can_parse_comment_token_with_escpaed_chars() {
        let token = SgfToken::from_pair("C", "a [wrapped\\] comment");
        assert_eq!(
            token,
            SgfToken::Comment("a [wrapped\\] comment".to_string())
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "C[a [wrapped\\] comment]");
    }

    #[test]
    fn can_parse_game_name_tokens() {
        let token = SgfToken::from_pair("GN", "game name");
        assert_eq!(token, SgfToken::GameName("game name".to_string()));
        let string_token: String = token.into();
        assert_eq!(string_token, "GN[game name]");
    }

    #[test]
    fn can_parse_copyright_tokens() {
        let token = SgfToken::from_pair("CR", "copyright");
        assert_eq!(token, SgfToken::Copyright("copyright".to_string()));
        let string_token: String = token.into();
        assert_eq!(string_token, "CR[copyright]");
    }

    #[test]
    fn can_parse_date_tokens() {
        let token = SgfToken::from_pair("DT", "2019-02-02");
        assert_eq!(token, SgfToken::Date("2019-02-02".to_string()));
        let string_token: String = token.into();
        assert_eq!(string_token, "DT[2019-02-02]");
    }

    #[test]
    fn can_parse_place_tokens() {
        let token = SgfToken::from_pair("PC", "place");
        assert_eq!(token, SgfToken::Place("place".to_string()));
        let string_token: String = token.into();
        assert_eq!(string_token, "PC[place]");
    }

    #[test]
    fn can_parse_mark_triangle_tokens() {
        let token = SgfToken::from_pair("TR", "aa");
        assert_eq!(token, SgfToken::Triangle { coordinate: (1, 1) });
        let string_token: String = token.into();
        assert_eq!(string_token, "TR[aa]");
    }

    #[test]
    fn can_parse_mark_square_tokens() {
        let token = SgfToken::from_pair("SQ", "aa");
        assert_eq!(token, SgfToken::Square { coordinate: (1, 1) });
        let string_token: String = token.into();
        assert_eq!(string_token, "SQ[aa]");
    }

    #[test]
    fn can_parse_mark_label_tokens() {
        let token = SgfToken::from_pair("LB", "kk:foo");
        assert_eq!(
            token,
            SgfToken::Label {
                label: "foo".to_string(),
                coordinate: (11, 11)
            }
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "LB[kk:foo]");
    }

    #[test]
    fn can_parse_add_tokens() {
        let token = SgfToken::from_pair("AB", "aa");
        assert_eq!(
            token,
            SgfToken::Add {
                color: Color::Black,
                coordinate: (1, 1),
            }
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "AB[aa]");

        let token = SgfToken::from_pair("AW", "kk");
        assert_eq!(
            token,
            SgfToken::Add {
                color: Color::White,
                coordinate: (11, 11),
            }
        );
        let string_token: String = token.into();
        assert_eq!(string_token, "AW[kk]");
    }
}
