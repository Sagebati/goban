use pest::Parser;

use pest::iterators::Pair;
use pest_derive::*;

use crate::*;

#[derive(Parser)]
#[grammar = "../sgf.pest"]
struct SGFParser;

///
/// Main entry point to the library. Parses an SGF string, and returns a `GameTree`.
///
/// Returns an `SgfError` when parsing failed, but it tries to recover from most kind of invalid input and insert `SgfToken::Invalid` or `SgfToken::Unknown` rather than failing
///
/// ```rust
/// use sgf_parser::*;
///
/// let tree: Result<GameTree, SgfError> = parse("(;EV[event]PB[black]PW[white]C[comment];B[aa];W[bb])");
///
/// let tree = tree.unwrap();
/// assert_eq!(tree.count_max_nodes(), 3);
/// ```
///
pub fn parse(input: &str) -> Result<GameTree, SgfError> {
    let mut parse_roots =
        SGFParser::parse(Rule::game_tree, input).map_err(SgfError::parse_error)?;
    if let Some(game_tree) = parse_roots.next() {
        let tree = parse_pair(game_tree);
        let game = create_game_tree(tree, true)?;
        Ok(game)
    } else {
        Ok(GameTree::default())
    }
}

/// Creates a `GameTree` from the Pest result
fn create_game_tree(parser_node: ParserNode<'_>, is_root: bool) -> Result<GameTree, SgfError> {
    if let ParserNode::GameTree(tree_nodes) = parser_node {
        let mut nodes: Vec<GameNode> = vec![];
        let mut variations: Vec<GameTree> = vec![];
        for node in tree_nodes {
            match node {
                ParserNode::Sequence(sequence_nodes) => {
                    nodes.extend(parse_sequence(sequence_nodes)?)
                }
                ParserNode::GameTree(_) => {
                    variations.push(create_game_tree(node, false)?);
                }
                _ => {
                    return Err(SgfErrorKind::ParseError.into());
                }
            }
        }
        let mut iter = nodes.iter();
        if is_root {
            iter.next();
        }
        let in_valid = iter.any(|node| node.tokens.iter().any(|token| token.is_root_token()));
        if in_valid {
            Err(SgfErrorKind::InvalidRootTokenPlacement.into())
        } else {
            Ok(GameTree { nodes, variations })
        }
    } else {
        Err(SgfErrorKind::ParseError.into())
    }
}

/// Parses a sequence of nodes to be added to a `GameTree`
fn parse_sequence(sequence_nodes: Vec<ParserNode<'_>>) -> Result<Vec<GameNode>, SgfError> {
    let mut nodes = vec![];
    for sequence_node in &sequence_nodes {
        if let ParserNode::Node(node_tokens) = sequence_node {
            let mut tokens: Vec<SgfToken> = vec![];
            for t in node_tokens {
                if let ParserNode::Token(token) = t {
                    tokens.push(token.clone());
                } else {
                    return Err(SgfErrorKind::ParseError.into());
                }
            }
            nodes.push(GameNode { tokens });
        } else {
            return Err(SgfErrorKind::ParseError.into());
        }
    }
    Ok(nodes)
}

/// Intermediate nodes from parsing the SGF file
#[derive(Debug, PartialEq, Clone)]
enum ParserNode<'a> {
    Token(SgfToken),
    Text(&'a str),
    Node(Vec<ParserNode<'a>>),
    Sequence(Vec<ParserNode<'a>>),
    GameTree(Vec<ParserNode<'a>>),
}

fn parse_pair(pair: Pair<'_, Rule>) -> ParserNode<'_> {
    match pair.as_rule() {
        Rule::game_tree => ParserNode::GameTree(pair.into_inner().map(parse_pair).collect()),
        Rule::sequence => ParserNode::Sequence(pair.into_inner().map(parse_pair).collect()),
        Rule::node => ParserNode::Node(pair.into_inner().map(parse_pair).collect()),
        Rule::property => {
            let text_nodes = pair.into_inner().map(parse_pair).collect::<Vec<_>>();
            let (ident, value) = match &text_nodes[..] {
                [ParserNode::Text(i), ParserNode::Text(v)] => (i, v),
                _ => {
                    panic!("Property node should only contain two text nodes");
                }
            };
            ParserNode::Token(SgfToken::from_pair(ident, value))
        }
        Rule::property_identifier => ParserNode::Text(pair.as_str()),
        Rule::property_value => {
            let value = pair.as_str();
            let end = value.len() - 1;
            ParserNode::Text(&value[1..end])
        }
        Rule::inner => {
            unreachable!();
        }
        Rule::char => {
            unreachable!();
        }
        Rule::WHITESPACE => {
            unreachable!();
        }
    }
}
