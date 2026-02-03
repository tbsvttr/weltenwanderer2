use chumsky::input::{Stream, ValueInput};
use chumsky::prelude::*;

use crate::ast::*;
use crate::lexer::Token;

type Span = SimpleSpan;

/// Parse error with source span.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: std::ops::Range<usize>,
    pub message: String,
}

fn to_ast_span(s: Span) -> crate::ast::Span {
    s.into_range()
}

fn spanned<T>(node: T, span: Span) -> Spanned<T> {
    Spanned {
        node,
        span: to_ast_span(span),
    }
}

fn is_direction(w: &str) -> bool {
    matches!(
        w,
        "north"
            | "south"
            | "east"
            | "west"
            | "up"
            | "down"
            | "northeast"
            | "northwest"
            | "southeast"
            | "southwest"
            | "out"
    )
}

/// Build the full source-file parser.
///
/// All sub-parsers are defined inline so chumsky can infer the generic input type.
fn source_file_parser<'a, I>() -> impl Parser<'a, I, SourceFile, extra::Err<Rich<'a, Token>>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    // -- Helpers --

    let kw = |k: &'static str| {
        select! { Token::Word(ref w) if w.as_str() == k => () }.labelled(k)
    };
    let word = select! { Token::Word(w) => w }.labelled("word");
    let string_lit = select! { Token::Str(s) => s }.labelled("string");
    let integer = select! { Token::Integer(n) => n }.labelled("integer");
    let float_lit = select! { Token::Float(n) => n }.labelled("float");
    let doc_string = select! { Token::DocString(s) => s }.labelled("doc string");

    // Zero or more newlines
    let nl = just(Token::Newline).repeated().to(());
    // One or more newlines
    let nl1 = just(Token::Newline).repeated().at_least(1).to(());

    // -- Entity name (in reference position) --
    let name_ref = choice((
        string_lit.map_with(|s, e| spanned(s, e.span())),
        word
            .repeated()
            .at_least(1)
            .collect::<Vec<String>>()
            .map_with(|words, e| spanned(words.join(" "), e.span())),
    ))
    .labelled("entity name");

    // -- Entity name in lists (same as name_ref) --
    let name_in_list = choice((
        string_lit.map_with(|s, e| spanned(s, e.span())),
        word
            .repeated()
            .at_least(1)
            .collect::<Vec<String>>()
            .map_with(|words, e| spanned(words.join(" "), e.span())),
    ))
    .labelled("entity name");

    // -- Entity name in declaration (words before "is") --
    let name_word = select! { Token::Word(ref w) if w.as_str() != "is" => w.clone() };
    let decl_name = choice((
        string_lit.map_with(|s, e| spanned(s, e.span())),
        name_word
            .repeated()
            .at_least(1)
            .collect::<Vec<String>>()
            .map_with(|words, e| spanned(words.join(" "), e.span())),
    ))
    .labelled("entity name");

    // -- Value --
    let value = recursive(|value| {
        let list = value
            .separated_by(just(Token::Comma).then(nl.clone()))
            .allow_trailing()
            .collect::<Vec<Spanned<Value>>>()
            .delimited_by(
                just(Token::LBracket).then(nl.clone()),
                nl.clone().then(just(Token::RBracket)),
            )
            .map(Value::List);

        choice((
            string_lit.map(Value::String),
            float_lit.map(Value::Float),
            integer.map(Value::Integer),
            kw("true").to(Value::Boolean(true)),
            kw("false").to(Value::Boolean(false)),
            list,
            word.map(Value::Identifier),
        ))
        .map_with(|v, e| spanned(v, e.span()))
        .labelled("value")
    });

    // -- Statements --

    // Relationship: "in <name>"
    let rel_in = kw("in")
        .ignore_then(name_ref)
        .map(|target| {
            Statement::Relationship(RelationshipStmt {
                keyword: RelationshipKeyword::In,
                targets: vec![target],
            })
        })
        .labelled("containment");

    // Two-word relationships (member of, located at, etc.)
    let rel_two = |first: &'static str, second: &'static str, keyword: RelationshipKeyword| {
        kw(first)
            .then(kw(second))
            .ignore_then(name_ref)
            .map(move |target| {
                Statement::Relationship(RelationshipStmt {
                    keyword: keyword.clone(),
                    targets: vec![target],
                })
            })
    };

    // List relationships (involving [...], references [...])
    let rel_list = |keyword_str: &'static str, keyword: RelationshipKeyword| {
        kw(keyword_str)
            .ignore_then(
                name_in_list
                    .separated_by(just(Token::Comma).then(nl.clone()))
                    .at_least(1)
                    .collect::<Vec<Spanned<String>>>()
                    .delimited_by(
                        just(Token::LBracket).then(nl.clone()),
                        nl.clone().then(just(Token::RBracket)),
                    ),
            )
            .map(move |targets| {
                Statement::Relationship(RelationshipStmt {
                    keyword: keyword.clone(),
                    targets,
                })
            })
    };

    let relationship = choice((
        rel_in,
        rel_two("member", "of", RelationshipKeyword::MemberOf),
        rel_two("located", "at", RelationshipKeyword::LocatedAt),
        rel_two("allied", "with", RelationshipKeyword::AlliedWith),
        rel_two("rival", "of", RelationshipKeyword::RivalOf),
        rel_two("owned", "by", RelationshipKeyword::OwnedBy),
        rel_two("led", "by", RelationshipKeyword::LedBy),
        rel_two("based", "at", RelationshipKeyword::BasedAt),
        rel_two("caused", "by", RelationshipKeyword::CausedBy),
        rel_list("involving", RelationshipKeyword::Involving),
        rel_list("references", RelationshipKeyword::References),
    ))
    .labelled("relationship");

    // Exit: "north to <name>"
    let exit_stmt = select! { Token::Word(ref w) if is_direction(w.as_str()) => w.clone() }
        .then_ignore(kw("to"))
        .then(name_ref)
        .map(|(direction, target)| Statement::Exit(ExitStmt { direction, target }))
        .labelled("exit");

    // Date: "date year -1247, month 3, day 15, era "Third Age""
    let date_field = choice((
        kw("year").ignore_then(integer).map(|n| {
            let mut d = DateLiteral::default();
            d.year = Some(n);
            d
        }),
        kw("month").ignore_then(integer).map(|n| {
            let mut d = DateLiteral::default();
            d.month = Some(n as u32);
            d
        }),
        kw("day").ignore_then(integer).map(|n| {
            let mut d = DateLiteral::default();
            d.day = Some(n as u32);
            d
        }),
        kw("era").ignore_then(string_lit.clone()).map(|s| {
            let mut d = DateLiteral::default();
            d.era = Some(s);
            d
        }),
    ));

    let date_stmt = kw("date")
        .ignore_then(
            date_field
                .separated_by(just(Token::Comma).then(nl.clone()))
                .at_least(1)
                .collect::<Vec<DateLiteral>>(),
        )
        .map(|fields| {
            let mut date = DateLiteral::default();
            for f in fields {
                if f.year.is_some() { date.year = f.year; }
                if f.month.is_some() { date.month = f.month; }
                if f.day.is_some() { date.day = f.day; }
                if f.era.is_some() { date.era = f.era; }
            }
            Statement::Date(date)
        })
        .labelled("date");

    // Description: """..."""
    let description = doc_string
        .map(Statement::Description)
        .labelled("description");

    // Property: word value
    let property = word
        .then(value)
        .map(|(key, val)| Statement::Property(Property { key, value: val.node }))
        .labelled("property");

    // Statement: try alternatives in order
    let statement = choice((
        relationship,
        exit_stmt,
        date_stmt,
        description,
        property,
    ))
    .map_with(|stmt, e| spanned(stmt, e.span()));

    // -- Block body: statements inside { } --
    let block_body = statement
        .separated_by(nl1.clone())
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(
            just(Token::LBrace).then(nl.clone()),
            nl.clone().then(just(Token::RBrace)),
        );

    // -- Top-level declarations --

    // world "Name" { ... }
    let world_decl = kw("world")
        .ignore_then(string_lit.map_with(|s, e| spanned(s, e.span())))
        .then(block_body.clone())
        .map(|(name, body)| Declaration::World(WorldDecl { name, body }))
        .labelled("world declaration");

    // <Name> is [a|an] <kind> { ... }
    let article = choice((kw("a"), kw("an"))).or_not();
    let entity_decl = decl_name
        .then_ignore(kw("is"))
        .then_ignore(article)
        .then(word.map_with(|w, e| spanned(w, e.span())))
        .then(block_body)
        .map(|((name, kind), body)| Declaration::Entity(EntityDecl { name, kind, body }))
        .labelled("entity declaration");

    let declaration = choice((world_decl, entity_decl))
        .map_with(|decl, e| spanned(decl, e.span()));

    // -- File --
    declaration
        .separated_by(nl1)
        .allow_trailing()
        .collect::<Vec<_>>()
        .padded_by(nl)
        .then_ignore(end())
        .map(|declarations| SourceFile { declarations })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse a token stream into an AST.
pub fn parse(
    tokens: &[(Token, std::ops::Range<usize>)],
) -> Result<SourceFile, Vec<ParseError>> {
    let token_iter = tokens
        .iter()
        .map(|(tok, span)| (tok.clone(), Span::from(span.clone())));

    let len = tokens.last().map_or(0, |(_, s)| s.end);
    let eoi: Span = (len..len).into();
    let stream = Stream::from_iter(token_iter)
        .map(eoi, |(t, s): (_, _)| (t, s));

    let (output, errors) = source_file_parser()
        .parse(stream)
        .into_output_errors();

    if let Some(ast) = output
        && errors.is_empty() {
            return Ok(ast);
        }

    Err(errors
        .into_iter()
        .map(|e| {
            let span = e.span();
            ParseError {
                span: span.into_range(),
                message: e.to_string(),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    fn parse_source(source: &str) -> Result<SourceFile, Vec<ParseError>> {
        let (tokens, lex_errors) = lexer::lex(source);
        assert!(lex_errors.is_empty(), "lex errors: {lex_errors:?}");
        parse(&tokens)
    }

    #[test]
    fn parse_world_declaration() {
        let ast = parse_source(
            r#"world "The Iron Kingdoms" {
    genre "high fantasy"
    setting "A shattered continent"
}"#,
        )
        .unwrap();

        assert_eq!(ast.declarations.len(), 1);
        match &ast.declarations[0].node {
            Declaration::World(w) => {
                assert_eq!(w.name.node, "The Iron Kingdoms");
                assert_eq!(w.body.len(), 2);
            }
            _ => panic!("expected world declaration"),
        }
    }

    #[test]
    fn parse_entity_declaration() {
        let ast = parse_source(
            "Kael Stormborn is a character {\n    species human\n    status alive\n}",
        )
        .unwrap();

        assert_eq!(ast.declarations.len(), 1);
        match &ast.declarations[0].node {
            Declaration::Entity(e) => {
                assert_eq!(e.name.node, "Kael Stormborn");
                assert_eq!(e.kind.node, "character");
                assert_eq!(e.body.len(), 2);
            }
            _ => panic!("expected entity declaration"),
        }
    }

    #[test]
    fn parse_entity_with_article_an() {
        let ast =
            parse_source("the Great Sundering is an event {\n    type cataclysm\n}").unwrap();

        match &ast.declarations[0].node {
            Declaration::Entity(e) => {
                assert_eq!(e.name.node, "the Great Sundering");
                assert_eq!(e.kind.node, "event");
            }
            _ => panic!("expected entity declaration"),
        }
    }

    #[test]
    fn parse_relationship_member_of() {
        let ast = parse_source("Kael is a character {\n    member of the Order of Dawn\n}")
            .unwrap();

        match &ast.declarations[0].node {
            Declaration::Entity(e) => match &e.body[0].node {
                Statement::Relationship(r) => {
                    assert_eq!(r.keyword, RelationshipKeyword::MemberOf);
                    assert_eq!(r.targets[0].node, "the Order of Dawn");
                }
                other => panic!("expected relationship, got {other:?}"),
            },
            _ => panic!("expected entity declaration"),
        }
    }

    #[test]
    fn parse_relationship_in() {
        let ast =
            parse_source("the Citadel is a fortress {\n    in the Ashlands\n}").unwrap();

        match &ast.declarations[0].node {
            Declaration::Entity(e) => match &e.body[0].node {
                Statement::Relationship(r) => {
                    assert_eq!(r.keyword, RelationshipKeyword::In);
                    assert_eq!(r.targets[0].node, "the Ashlands");
                }
                other => panic!("expected relationship, got {other:?}"),
            },
            _ => panic!("expected entity declaration"),
        }
    }

    #[test]
    fn parse_exit() {
        let ast = parse_source("the Citadel is a fortress {\n    north to the Ashlands\n}")
            .unwrap();

        match &ast.declarations[0].node {
            Declaration::Entity(e) => match &e.body[0].node {
                Statement::Exit(exit) => {
                    assert_eq!(exit.direction, "north");
                    assert_eq!(exit.target.node, "the Ashlands");
                }
                other => panic!("expected exit, got {other:?}"),
            },
            _ => panic!("expected entity declaration"),
        }
    }

    #[test]
    fn parse_list_value() {
        let ast = parse_source("Kael is a character {\n    traits [brave, stubborn, loyal]\n}")
            .unwrap();

        match &ast.declarations[0].node {
            Declaration::Entity(e) => match &e.body[0].node {
                Statement::Property(p) => {
                    assert_eq!(p.key, "traits");
                    match &p.value {
                        Value::List(items) => assert_eq!(items.len(), 3),
                        other => panic!("expected list, got {other:?}"),
                    }
                }
                other => panic!("expected property, got {other:?}"),
            },
            _ => panic!("expected entity declaration"),
        }
    }

    #[test]
    fn parse_date() {
        let ast =
            parse_source("the Sundering is an event {\n    date year -1247, month 3, day 15\n}")
                .unwrap();

        match &ast.declarations[0].node {
            Declaration::Entity(e) => match &e.body[0].node {
                Statement::Date(d) => {
                    assert_eq!(d.year, Some(-1247));
                    assert_eq!(d.month, Some(3));
                    assert_eq!(d.day, Some(15));
                }
                other => panic!("expected date, got {other:?}"),
            },
            _ => panic!("expected entity declaration"),
        }
    }

    #[test]
    fn parse_date_with_era() {
        let ast = parse_source(
            "the Sundering is an event {\n    date year -1247, month 3, day 15, era \"Third Age\"\n}",
        )
        .unwrap();

        match &ast.declarations[0].node {
            Declaration::Entity(e) => match &e.body[0].node {
                Statement::Date(d) => {
                    assert_eq!(d.year, Some(-1247));
                    assert_eq!(d.month, Some(3));
                    assert_eq!(d.day, Some(15));
                    assert_eq!(d.era.as_deref(), Some("Third Age"));
                }
                other => panic!("expected date, got {other:?}"),
            },
            _ => panic!("expected entity declaration"),
        }
    }

    #[test]
    fn parse_description() {
        let source = "Kael is a character {\n    \"\"\"\n    A brave knight.\n    \"\"\"\n}";
        let ast = parse_source(source).unwrap();

        match &ast.declarations[0].node {
            Declaration::Entity(e) => match &e.body[0].node {
                Statement::Description(text) => {
                    assert!(text.contains("brave knight"));
                }
                other => panic!("expected description, got {other:?}"),
            },
            _ => panic!("expected entity declaration"),
        }
    }

    #[test]
    fn parse_involving_list() {
        let ast = parse_source(
            "the Battle is an event {\n    involving [Kael Stormborn, the Order of Dawn]\n}",
        )
        .unwrap();

        match &ast.declarations[0].node {
            Declaration::Entity(e) => match &e.body[0].node {
                Statement::Relationship(r) => {
                    assert_eq!(r.keyword, RelationshipKeyword::Involving);
                    assert_eq!(r.targets.len(), 2);
                    assert_eq!(r.targets[0].node, "Kael Stormborn");
                    assert_eq!(r.targets[1].node, "the Order of Dawn");
                }
                other => panic!("expected relationship, got {other:?}"),
            },
            _ => panic!("expected entity declaration"),
        }
    }

    #[test]
    fn parse_multiple_declarations() {
        let source = r#"world "Test" {
    genre "fantasy"
}

Kael is a character {
    species human
}

the Citadel is a fortress {
    population 45000
}"#;
        let ast = parse_source(source).unwrap();
        assert_eq!(ast.declarations.len(), 3);
    }

    #[test]
    fn parse_integer_property() {
        let ast = parse_source("the Citadel is a fortress {\n    population 45000\n}")
            .unwrap();

        match &ast.declarations[0].node {
            Declaration::Entity(e) => match &e.body[0].node {
                Statement::Property(p) => {
                    assert_eq!(p.key, "population");
                    assert!(matches!(p.value, Value::Integer(45000)));
                }
                other => panic!("expected property, got {other:?}"),
            },
            _ => panic!("expected entity declaration"),
        }
    }
}
