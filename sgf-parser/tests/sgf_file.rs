#[cfg(test)]
mod sgf_files_test {
    #[test]
    fn parse_sfg() {
        let _g = sgf_parser::parse(include_str!("sgf/ShusakuvsInseki.sgf")).unwrap();
    }
}
