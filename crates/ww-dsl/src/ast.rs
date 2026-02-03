/// Source span as a byte range.
pub type Span = std::ops::Range<usize>;

/// An AST node with source location.
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

/// A parsed source file.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub declarations: Vec<Spanned<Declaration>>,
}

#[derive(Debug, Clone)]
pub enum Declaration {
    World(WorldDecl),
    Entity(EntityDecl),
}

#[derive(Debug, Clone)]
pub struct WorldDecl {
    pub name: Spanned<String>,
    pub body: Vec<Spanned<Statement>>,
}

#[derive(Debug, Clone)]
pub struct EntityDecl {
    pub name: Spanned<String>,
    pub kind: Spanned<String>,
    pub body: Vec<Spanned<Statement>>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Property(Property),
    Relationship(RelationshipStmt),
    Exit(ExitStmt),
    Description(String),
    Date(DateLiteral),
}

#[derive(Debug, Clone)]
pub struct Property {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Identifier(String),
    List(Vec<Spanned<Value>>),
}

#[derive(Debug, Clone)]
pub struct RelationshipStmt {
    pub keyword: RelationshipKeyword,
    pub targets: Vec<Spanned<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipKeyword {
    In,
    MemberOf,
    LocatedAt,
    AlliedWith,
    RivalOf,
    OwnedBy,
    LedBy,
    BasedAt,
    Involving,
    References,
    CausedBy,
}

#[derive(Debug, Clone)]
pub struct ExitStmt {
    pub direction: String,
    pub target: Spanned<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DateLiteral {
    pub year: Option<i64>,
    pub month: Option<u32>,
    pub day: Option<u32>,
    pub era: Option<String>,
}
