/// Source span as a byte range.
pub type Span = std::ops::Range<usize>;

/// An AST node with source location.
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    /// The wrapped AST node.
    pub node: T,
    /// The byte range of this node in the source text.
    pub span: Span,
}

/// A parsed source file.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Top-level declarations in the source file.
    pub declarations: Vec<Spanned<Declaration>>,
}

/// A top-level declaration in the DSL.
#[derive(Debug, Clone)]
pub enum Declaration {
    /// A world declaration defining a setting or realm.
    World(WorldDecl),
    /// An entity declaration defining a character, location, or other object.
    Entity(EntityDecl),
}

/// A world declaration, e.g. `world "Iron Kingdoms" { ... }`.
#[derive(Debug, Clone)]
pub struct WorldDecl {
    /// The name of the world.
    pub name: Spanned<String>,
    /// The statements contained in the world body.
    pub body: Vec<Spanned<Statement>>,
}

/// An entity declaration, e.g. `Kael is a character { ... }`.
#[derive(Debug, Clone)]
pub struct EntityDecl {
    /// The name of the entity.
    pub name: Spanned<String>,
    /// Inline relationship annotations, e.g. `Kael (leader of the Order) is a ...`.
    pub annotations: Vec<Spanned<InlineAnnotation>>,
    /// The kind of entity (e.g. "character", "location", "faction").
    pub kind: Spanned<String>,
    /// The statements contained in the entity body.
    pub body: Vec<Spanned<Statement>>,
}

/// A statement within a declaration body.
#[derive(Debug, Clone)]
pub enum Statement {
    /// A key-value property assignment.
    Property(Property),
    /// A relationship to other entities.
    Relationship(RelationshipStmt),
    /// A directional exit to another location.
    Exit(ExitStmt),
    /// A freeform text description.
    Description(String),
    /// A date literal value.
    Date(DateLiteral),
    /// A named block grouping related statements.
    Block(BlockStmt),
}

/// A key-value property, e.g. `population: 15000`.
#[derive(Debug, Clone)]
pub struct Property {
    /// The property name.
    pub key: String,
    /// The property value.
    pub value: Value,
}

/// A property value in the DSL.
#[derive(Debug, Clone)]
pub enum Value {
    /// A quoted string literal.
    String(String),
    /// An integer literal.
    Integer(i64),
    /// A floating-point literal.
    Float(f64),
    /// A boolean literal (`true` or `false`).
    Boolean(bool),
    /// An unquoted identifier reference.
    Identifier(String),
    /// A list of values, e.g. `[a, b, c]`.
    List(Vec<Spanned<Value>>),
}

/// A relationship statement, e.g. `member of "The Order"`.
#[derive(Debug, Clone)]
pub struct RelationshipStmt {
    /// The relationship keyword (e.g. `member of`, `in`).
    pub keyword: RelationshipKeyword,
    /// The target entities of the relationship.
    pub targets: Vec<Spanned<String>>,
}

/// A keyword identifying the type of relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipKeyword {
    /// Containment relationship (`in`).
    In,
    /// Membership relationship (`member of`).
    MemberOf,
    /// Location relationship (`located at`).
    LocatedAt,
    /// Alliance relationship (`allied with`).
    AlliedWith,
    /// Rivalry relationship (`rival of`).
    RivalOf,
    /// Ownership relationship (`owned by`).
    OwnedBy,
    /// Leadership relationship (`led by`).
    LedBy,
    /// Basing relationship (`based at`).
    BasedAt,
    /// Involvement relationship (`involving`).
    Involving,
    /// Reference relationship (`references`).
    References,
    /// Causation relationship (`caused by`).
    CausedBy,
}

/// A directional exit statement, e.g. `exit north to "The Citadel"`.
#[derive(Debug, Clone)]
pub struct ExitStmt {
    /// The direction of the exit (e.g. "north", "south").
    pub direction: String,
    /// The target location this exit leads to.
    pub target: Spanned<String>,
}

/// A named block grouping properties under a namespace prefix.
#[derive(Debug, Clone)]
pub struct BlockStmt {
    /// The block name used as a namespace prefix.
    pub name: String,
    /// Optional string argument, e.g. `dialogue "greeting" { ... }`.
    pub arg: Option<String>,
    /// The statements contained in the block.
    pub body: Vec<Spanned<Statement>>,
}

/// An inline relationship annotation on an entity declaration.
///
/// E.g. `Kael (leader of the Order) is a character { ... }`.
#[derive(Debug, Clone)]
pub struct InlineAnnotation {
    /// The relationship keyword for this annotation.
    pub keyword: RelationshipKeyword,
    /// The target entities referenced by the annotation.
    pub targets: Vec<Spanned<String>>,
}

/// A date literal, e.g. `1247-03-15 Third Age`.
#[derive(Debug, Clone, Default)]
pub struct DateLiteral {
    /// The year component, if present.
    pub year: Option<i64>,
    /// The month component (1-12), if present.
    pub month: Option<u32>,
    /// The day component (1-31), if present.
    pub day: Option<u32>,
    /// The era or calendar name, if present.
    pub era: Option<String>,
}
