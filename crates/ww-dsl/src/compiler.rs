use ww_core::component::*;
use ww_core::entity::{Entity, EntityId, EntityKind, MetadataValue};
use ww_core::relationship::{Relationship, RelationshipKind};
use ww_core::world::{World, WorldMeta};

use crate::ast::*;
use crate::diagnostics::{Diagnostic, Severity};
use crate::resolver::{Resolver, SourceMap};

/// Result of compiling DSL source into a World.
pub struct CompileResult {
    /// The compiled world (may be partial if errors occurred).
    pub world: World,
    /// Errors and warnings produced during compilation.
    pub diagnostics: Vec<Diagnostic>,
    /// Maps byte offsets back to individual source files.
    pub source_map: SourceMap,
}

impl CompileResult {
    /// Returns `true` if any diagnostic has error severity.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
}

/// Compile a parsed AST into a ww-core World.
///
/// The compilation happens in two passes:
/// 1. **Entity pass**: create all entities using IDs pre-assigned by the resolver
/// 2. **Relationship pass**: resolve name references via the resolver and create relationships
pub fn compile(ast: &SourceFile, resolver: &Resolver, source_map: SourceMap) -> CompileResult {
    let mut compiler = Compiler::new(resolver, &source_map);
    compiler.compile(ast);
    // Merge resolver diagnostics (duplicates) first, then compiler diagnostics
    let mut diagnostics = resolver.diagnostics.clone();
    diagnostics.append(&mut compiler.diagnostics);
    CompileResult {
        world: compiler.world,
        diagnostics,
        source_map,
    }
}

struct Compiler<'a> {
    world: World,
    diagnostics: Vec<Diagnostic>,
    resolver: &'a Resolver,
    source_map: &'a SourceMap,
    ast: Option<&'a SourceFile>,
}

impl<'a> Compiler<'a> {
    fn new(resolver: &'a Resolver, source_map: &'a SourceMap) -> Self {
        Self {
            world: World::new(WorldMeta::new("Untitled")),
            diagnostics: Vec::new(),
            resolver,
            source_map,
            ast: None,
        }
    }

    fn compile(&mut self, ast: &'a SourceFile) {
        self.ast = Some(ast);

        // Pass 1: process world metadata and create entities
        for decl in &ast.declarations {
            match &decl.node {
                Declaration::World(w) => self.compile_world_meta(w),
                Declaration::Entity(e) => self.compile_entity_pass1(e),
            }
        }

        // Pass 2: process relationships and exits (name references are now resolvable)
        for decl in &ast.declarations {
            if let Declaration::Entity(e) = &decl.node {
                self.compile_entity_pass2(e);
            }
        }
    }

    // -- Pass 1: World metadata and entity creation --

    fn compile_world_meta(&mut self, decl: &WorldDecl) {
        self.world.meta.name = decl.name.node.clone();

        for stmt in &decl.body {
            match &stmt.node {
                Statement::Property(prop) => match prop.key.as_str() {
                    "genre" => {
                        if let Some(s) = self.value_as_string(&prop.value) {
                            self.world.meta.genre = Some(s);
                        }
                    }
                    "setting" => {
                        if let Some(s) = self.value_as_string(&prop.value) {
                            self.world.meta.setting = Some(s);
                        }
                    }
                    "description" => {
                        if let Some(s) = self.value_as_string(&prop.value) {
                            self.world.meta.description = s;
                        }
                    }
                    other => {
                        let mv = self.value_to_metadata(&prop.value);
                        self.world.meta.properties.insert(other.to_string(), mv);
                    }
                },
                Statement::Block(block) => {
                    for inner in &block.body {
                        if let Statement::Property(prop) = &inner.node {
                            let key = format!("{}.{}", block.name, prop.key);
                            let mv = self.value_to_metadata(&prop.value);
                            self.world.meta.properties.insert(key, mv);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn compile_entity_pass1(&mut self, decl: &EntityDecl) {
        // Skip entities the resolver flagged as duplicates
        let resolved = match self.resolver.get(&decl.name.node) {
            Some(r) => r,
            None => return,
        };

        let name_lower = decl.name.node.to_lowercase();

        // Resolve entity kind — may traverse inheritance chain
        let (kind, location_subtype) = self.resolve_entity_kind(&name_lower, &decl.kind.node);
        let mut entity = Entity::with_id(resolved.id, kind, &decl.name.node);

        // Set location subtype if applicable
        if let Some(subtype) = location_subtype {
            entity.components.location = Some(LocationComponent {
                location_type: subtype,
                ..Default::default()
            });
        }

        // If inheriting, apply parent properties first as defaults
        if let Some(parent_lower) = self.resolver.inheritance.get(&name_lower).cloned() {
            self.apply_inherited_properties(&mut entity, &parent_lower, &mut Vec::new());
        }

        // Process properties, component fields, descriptions, and dates
        for stmt in &decl.body {
            match &stmt.node {
                Statement::Property(prop) => {
                    self.apply_property(&mut entity, prop, &stmt.span);
                }
                Statement::Description(text) => {
                    entity.description = text.clone();
                }
                Statement::Date(date) => {
                    self.apply_date(&mut entity, date);
                }
                Statement::Block(block) => {
                    self.apply_block_properties(&mut entity, &block.name, &block.body);
                }
                // Relationships handled in pass 2
                Statement::Relationship(_) | Statement::Exit(_) => {}
            }
        }

        if let Err(e) = self.world.add_entity(entity) {
            self.diagnostics
                .push(Diagnostic::error(decl.name.span.clone(), e.to_string()));
        }
    }

    // -- Pass 2: Relationships and exits --

    fn compile_entity_pass2(&mut self, decl: &EntityDecl) {
        let source_id = match self.resolve_name(&decl.name.node, &decl.name.span) {
            Some(id) => id,
            None => return,
        };

        // Process inline annotations as relationships
        for ann in &decl.annotations {
            let rel = RelationshipStmt {
                keyword: ann.node.keyword.clone(),
                targets: ann.node.targets.clone(),
            };
            self.compile_relationship(source_id, &rel);
        }

        for stmt in &decl.body {
            match &stmt.node {
                Statement::Relationship(rel) => {
                    self.compile_relationship(source_id, rel);
                }
                Statement::Exit(exit) => {
                    self.compile_exit(source_id, exit);
                }
                Statement::Block(_) => {} // Blocks contain only properties, handled in pass 1
                _ => {}                   // Already handled in pass 1
            }
        }
    }

    fn compile_relationship(&mut self, source_id: EntityId, rel: &RelationshipStmt) {
        let kind = match rel.keyword {
            RelationshipKeyword::In => RelationshipKind::ContainedIn,
            RelationshipKeyword::MemberOf => RelationshipKind::MemberOf,
            RelationshipKeyword::LocatedAt => RelationshipKind::LocatedAt,
            RelationshipKeyword::AlliedWith => RelationshipKind::AlliedWith,
            RelationshipKeyword::RivalOf => RelationshipKind::RivalOf,
            RelationshipKeyword::OwnedBy => RelationshipKind::OwnedBy,
            RelationshipKeyword::LedBy => RelationshipKind::LeaderOf,
            RelationshipKeyword::BasedAt => RelationshipKind::BasedAt,
            RelationshipKeyword::Involving => RelationshipKind::ParticipatedIn,
            RelationshipKeyword::References => RelationshipKind::References,
            RelationshipKeyword::CausedBy => RelationshipKind::CausedBy,
        };

        for target in &rel.targets {
            let target_id = match self.resolve_name(&target.node, &target.span) {
                Some(id) => id,
                None => continue,
            };

            // Handle inverted relationships:
            // "led by X" means X leads self, so X is source
            // "owned by X" means X owns self, so X is source
            // "involving [X, Y]" means X/Y participated in self
            let (src, tgt) = match rel.keyword {
                RelationshipKeyword::LedBy | RelationshipKeyword::OwnedBy => (target_id, source_id),
                RelationshipKeyword::Involving => (target_id, source_id),
                _ => (source_id, target_id),
            };

            let relationship = Relationship::new(src, kind.clone(), tgt);
            if let Err(e) = self.world.add_relationship(relationship) {
                self.diagnostics.push(Diagnostic::error(
                    target.span.clone(),
                    format!("failed to add relationship: {e}"),
                ));
            }
        }
    }

    fn compile_exit(&mut self, source_id: EntityId, exit: &ExitStmt) {
        let target_id = match self.resolve_name(&exit.target.node, &exit.target.span) {
            Some(id) => id,
            None => return,
        };

        let relationship = Relationship::new(source_id, RelationshipKind::ConnectedTo, target_id)
            .with_label(&exit.direction);

        if let Err(e) = self.world.add_relationship(relationship) {
            self.diagnostics.push(Diagnostic::error(
                exit.target.span.clone(),
                format!("failed to add exit: {e}"),
            ));
        }
    }

    // -- Inheritance --

    /// Resolve the actual EntityKind for an entity, walking the inheritance chain.
    fn resolve_entity_kind(
        &self,
        name_lower: &str,
        kind_str: &str,
    ) -> (EntityKind, Option<String>) {
        let kind_lower = kind_str.to_lowercase();

        // If the kind is not an entity name, parse it directly
        if !self.resolver.inheritance.contains_key(name_lower) {
            return EntityKind::parse(&kind_lower);
        }

        // Walk the inheritance chain to find the root kind
        let mut current = kind_lower;
        let mut visited = vec![name_lower.to_string()];
        loop {
            if visited.contains(&current) {
                // Cycle detected
                return (EntityKind::Custom(kind_str.to_string()), None);
            }
            visited.push(current.clone());

            // Check if this level has a parent too
            if let Some(parent) = self.resolver.inheritance.get(&current) {
                current = parent.clone();
            } else {
                // `current` is the root — parse its kind from its declaration
                if let Some(ast) = self.ast {
                    for decl in &ast.declarations {
                        if let Declaration::Entity(e) = &decl.node
                            && e.name.node.to_lowercase() == current
                        {
                            return EntityKind::parse(&e.kind.node.to_lowercase());
                        }
                    }
                }
                // Fallback: treat current as the kind string
                return EntityKind::parse(&current);
            }
        }
    }

    /// Apply inherited properties from a parent entity's declaration.
    fn apply_inherited_properties(
        &mut self,
        entity: &mut Entity,
        parent_lower: &str,
        visited: &mut Vec<String>,
    ) {
        if visited.contains(&parent_lower.to_string()) {
            // Cycle detected
            self.diagnostics.push(Diagnostic::error(
                0..0,
                format!("inheritance cycle detected involving \"{parent_lower}\""),
            ));
            return;
        }
        visited.push(parent_lower.to_string());

        // If parent also inherits, apply grandparent first
        if let Some(grandparent) = self.resolver.inheritance.get(parent_lower).cloned() {
            self.apply_inherited_properties(entity, &grandparent, visited);
        }

        // Find parent declaration in AST and apply its properties
        let Some(ast) = self.ast else { return };
        for decl in &ast.declarations {
            if let Declaration::Entity(parent_decl) = &decl.node
                && parent_decl.name.node.to_lowercase() == parent_lower
            {
                for stmt in &parent_decl.body {
                    match &stmt.node {
                        Statement::Property(prop) => {
                            self.apply_property(entity, prop, &stmt.span);
                        }
                        Statement::Description(text) => {
                            if entity.description.is_empty() {
                                entity.description = text.clone();
                            }
                        }
                        Statement::Date(date) => {
                            self.apply_date(entity, date);
                        }
                        Statement::Block(block) => {
                            self.apply_block_properties(entity, &block.name, &block.body);
                        }
                        // Don't inherit relationships or exits
                        Statement::Relationship(_) | Statement::Exit(_) => {}
                    }
                }
                break;
            }
        }
    }

    // -- Block property flattening --

    fn apply_block_properties(
        &mut self,
        entity: &mut Entity,
        prefix: &str,
        body: &[Spanned<Statement>],
    ) {
        for stmt in body {
            match &stmt.node {
                Statement::Property(prop) => {
                    let key = format!("{prefix}.{}", prop.key);
                    let mv = self.value_to_metadata(&prop.value);
                    entity.properties.insert(key, mv);
                }
                Statement::Block(inner) => {
                    let nested_prefix = format!("{prefix}.{}", inner.name);
                    self.apply_block_properties(entity, &nested_prefix, &inner.body);
                }
                Statement::Description(text) => {
                    let key = format!("{prefix}.description");
                    entity
                        .properties
                        .insert(key, MetadataValue::String(text.clone()));
                }
                Statement::Relationship(_) | Statement::Exit(_) => {
                    self.diagnostics.push(Diagnostic::warning(
                        stmt.span.clone(),
                        format!("relationships and exits are not allowed inside '{prefix}' block"),
                    ));
                }
                Statement::Date(_) => {
                    self.diagnostics.push(Diagnostic::warning(
                        stmt.span.clone(),
                        format!("dates are not allowed inside '{prefix}' block"),
                    ));
                }
            }
        }
    }

    // -- Property application --

    fn apply_property(&mut self, entity: &mut Entity, prop: &Property, span: &crate::ast::Span) {
        // Try to apply as a component field first
        if self.apply_component_property(entity, prop) {
            return;
        }

        // Otherwise store as a generic property
        let key = prop.key.clone();
        let mv = self.value_to_metadata(&prop.value);
        entity.properties.insert(key, mv);

        let _ = span; // span available for future diagnostics
    }

    /// Try to apply a property as a typed component field. Returns true if handled.
    fn apply_component_property(&mut self, entity: &mut Entity, prop: &Property) -> bool {
        match prop.key.as_str() {
            // Character fields
            "species" => {
                let comp = entity
                    .components
                    .character
                    .get_or_insert_with(Default::default);
                comp.species = self.value_as_string(&prop.value);
                true
            }
            "occupation" => {
                let comp = entity
                    .components
                    .character
                    .get_or_insert_with(Default::default);
                comp.occupation = self.value_as_string(&prop.value);
                true
            }
            "status" => {
                let comp = entity
                    .components
                    .character
                    .get_or_insert_with(Default::default);
                if let Some(s) = self.value_as_string(&prop.value) {
                    comp.status = match s.to_lowercase().as_str() {
                        "alive" => CharacterStatus::Alive,
                        "dead" => CharacterStatus::Dead,
                        "unknown" => CharacterStatus::Unknown,
                        _ => CharacterStatus::Custom(s),
                    };
                }
                true
            }
            "traits" => {
                let comp = entity
                    .components
                    .character
                    .get_or_insert_with(Default::default);
                if let Value::List(items) = &prop.value {
                    comp.traits = items
                        .iter()
                        .filter_map(|v| self.value_as_string(&v.node))
                        .collect();
                }
                true
            }

            // Location fields
            "climate" => {
                let comp = entity
                    .components
                    .location
                    .get_or_insert_with(Default::default);
                comp.climate = self.value_as_string(&prop.value);
                true
            }
            "terrain" => {
                let comp = entity
                    .components
                    .location
                    .get_or_insert_with(Default::default);
                comp.terrain = self.value_as_string(&prop.value);
                true
            }
            "population" => {
                let comp = entity
                    .components
                    .location
                    .get_or_insert_with(Default::default);
                if let Value::Integer(n) = &prop.value {
                    comp.population = Some(*n as u64);
                }
                true
            }

            // Faction fields
            "alignment" => {
                let comp = entity
                    .components
                    .faction
                    .get_or_insert_with(Default::default);
                comp.alignment = self.value_as_string(&prop.value);
                true
            }
            "values" => {
                let comp = entity
                    .components
                    .faction
                    .get_or_insert_with(Default::default);
                if let Value::List(items) = &prop.value {
                    comp.values = items
                        .iter()
                        .filter_map(|v| self.value_as_string(&v.node))
                        .collect();
                }
                true
            }

            // Event fields
            "outcome" => {
                let comp = entity.components.event.get_or_insert_with(Default::default);
                comp.outcome = self.value_as_string(&prop.value);
                true
            }
            "duration" => {
                let comp = entity.components.event.get_or_insert_with(Default::default);
                comp.duration = self.value_as_string(&prop.value);
                true
            }

            // Item fields
            "rarity" => {
                let comp = entity.components.item.get_or_insert_with(Default::default);
                comp.rarity = self.value_as_string(&prop.value);
                true
            }

            // Lore fields
            "source" => {
                let comp = entity.components.lore.get_or_insert_with(Default::default);
                comp.source = self.value_as_string(&prop.value);
                true
            }
            "reliability" => {
                let comp = entity.components.lore.get_or_insert_with(Default::default);
                comp.reliability = self.value_as_string(&prop.value);
                true
            }

            // "type" is polymorphic — applies to the relevant component
            "type" => {
                if let Some(s) = self.value_as_string(&prop.value) {
                    // Apply to whichever component exists, or store as generic
                    if let Some(comp) = &mut entity.components.event {
                        comp.event_type = Some(s);
                    } else if let Some(comp) = &mut entity.components.faction {
                        comp.faction_type = Some(s);
                    } else if let Some(comp) = &mut entity.components.item {
                        comp.item_type = Some(s);
                    } else if let Some(comp) = &mut entity.components.lore {
                        comp.lore_type = Some(s);
                    } else {
                        // Store as generic property
                        return false;
                    }
                }
                true
            }

            _ => false,
        }
    }

    fn apply_date(&mut self, entity: &mut Entity, date: &DateLiteral) {
        let comp = entity.components.event.get_or_insert_with(Default::default);
        let mut wd = WorldDate::new(date.year.unwrap_or(0));
        wd.month = date.month;
        wd.day = date.day;
        wd.era = date.era.clone();
        comp.date = Some(wd);
    }

    // -- Name resolution --

    fn resolve_name(&mut self, name: &str, span: &crate::ast::Span) -> Option<EntityId> {
        self.resolver
            .lookup(name, span, self.source_map, &mut self.diagnostics)
    }

    // -- Value conversion helpers --

    fn value_as_string(&self, value: &Value) -> Option<String> {
        match value {
            Value::String(s) => Some(s.clone()),
            Value::Identifier(s) => Some(s.clone()),
            Value::Integer(n) => Some(n.to_string()),
            Value::Float(n) => Some(n.to_string()),
            Value::Boolean(b) => Some(b.to_string()),
            Value::List(_) => None,
        }
    }

    fn value_to_metadata(&self, value: &Value) -> MetadataValue {
        match value {
            Value::String(s) => MetadataValue::String(s.clone()),
            Value::Identifier(s) => MetadataValue::String(s.clone()),
            Value::Integer(n) => MetadataValue::Integer(*n),
            Value::Float(n) => MetadataValue::Float(*n),
            Value::Boolean(b) => MetadataValue::Boolean(*b),
            Value::List(items) => MetadataValue::List(
                items
                    .iter()
                    .map(|v| self.value_to_metadata(&v.node))
                    .collect(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    use crate::parser;

    fn compile_source(source: &str) -> CompileResult {
        let (tokens, lex_errors) = lexer::lex(source);
        assert!(lex_errors.is_empty(), "lex errors: {lex_errors:?}");
        let ast = parser::parse(&tokens).expect("parse error");
        let source_map = SourceMap::single(source.len());
        let resolver = Resolver::resolve(&ast, &source_map);
        compile(&ast, &resolver, source_map)
    }

    #[test]
    fn compile_world_metadata() {
        let result = compile_source(
            r#"world "The Iron Kingdoms" {
    genre "high fantasy"
    setting "A shattered continent"
}"#,
        );
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);
        assert_eq!(result.world.meta.name, "The Iron Kingdoms");
        assert_eq!(result.world.meta.genre.as_deref(), Some("high fantasy"));
    }

    #[test]
    fn compile_character_entity() {
        let result = compile_source(
            r#"Kael Stormborn is a character {
    species human
    occupation knight
    status alive
    traits [brave, stubborn, loyal]
}"#,
        );
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);

        let entity = result.world.find_by_name("Kael Stormborn").unwrap();
        assert_eq!(entity.kind, EntityKind::Character);

        let char_comp = entity.components.character.as_ref().unwrap();
        assert_eq!(char_comp.species.as_deref(), Some("human"));
        assert_eq!(char_comp.occupation.as_deref(), Some("knight"));
        assert_eq!(char_comp.status, CharacterStatus::Alive);
        assert_eq!(char_comp.traits, vec!["brave", "stubborn", "loyal"]);
    }

    #[test]
    fn compile_location_with_subtype() {
        let result = compile_source(
            r#"the Iron Citadel is a fortress {
    climate arid
    population 45000
}"#,
        );
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);

        let entity = result.world.find_by_name("the Iron Citadel").unwrap();
        assert_eq!(entity.kind, EntityKind::Location);
        assert_eq!(entity.location_subtype(), Some("fortress"));

        let loc = entity.components.location.as_ref().unwrap();
        assert_eq!(loc.climate.as_deref(), Some("arid"));
        assert_eq!(loc.population, Some(45000));
    }

    #[test]
    fn compile_relationships() {
        let result = compile_source(
            r#"Kael is a character {
    species human
}

the Order of Dawn is a faction {
    type military_order
}

Kael is a character {
    member of the Order of Dawn
}"#,
        );
        // This should error because "Kael" is defined twice
        assert!(result.has_errors());
    }

    #[test]
    fn compile_relationships_no_duplicate() {
        let result = compile_source(
            r#"Kael is a character {
    species human
}

the Order of Dawn is a faction {
    type military_order
}

the Iron Citadel is a fortress {
    population 45000
}"#,
        );
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);
        assert_eq!(result.world.entity_count(), 3);
    }

    #[test]
    fn compile_member_of_relationship() {
        let result = compile_source(
            r#"Kael is a character {
    member of the Order of Dawn
}

the Order of Dawn is a faction {
    type military_order
}"#,
        );
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);

        let kael_id = result.world.find_id_by_name("Kael").unwrap();
        let rels = result.world.relationships_from(kael_id);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].kind, RelationshipKind::MemberOf);
    }

    #[test]
    fn compile_exit() {
        let result = compile_source(
            r#"the Citadel is a fortress {
    north to the Ashlands
}

the Ashlands is a region {
    climate arid
}"#,
        );
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);

        let citadel_id = result.world.find_id_by_name("the Citadel").unwrap();
        let rels = result.world.relationships_from(citadel_id);
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0].kind, RelationshipKind::ConnectedTo);
        assert_eq!(rels[0].label.as_deref(), Some("north"));
    }

    #[test]
    fn compile_event_with_date() {
        let result = compile_source(
            r#"the Great Sundering is an event {
    date year -1247, month 3, day 15
    type cataclysm

    """
    The day the world broke.
    """
}"#,
        );
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);

        let entity = result.world.find_by_name("the Great Sundering").unwrap();
        let event = entity.components.event.as_ref().unwrap();
        assert_eq!(event.date.as_ref().unwrap().year, -1247);
        assert_eq!(event.event_type.as_deref(), Some("cataclysm"));
        assert!(entity.description.contains("world broke"));
    }

    #[test]
    fn compile_event_with_date_and_era() {
        let result = compile_source(
            r#"the Great Sundering is an event {
    date year -1247, month 3, day 15, era "Third Age"
    type cataclysm
}"#,
        );
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);

        let entity = result.world.find_by_name("the Great Sundering").unwrap();
        let event = entity.components.event.as_ref().unwrap();
        let date = event.date.as_ref().unwrap();
        assert_eq!(date.year, -1247);
        assert_eq!(date.month, Some(3));
        assert_eq!(date.day, Some(15));
        assert_eq!(date.era.as_deref(), Some("Third Age"));
    }

    #[test]
    fn compile_undefined_reference_produces_error() {
        let result = compile_source(
            r#"Kael is a character {
    member of the Nonexistent Order
}"#,
        );
        assert!(result.has_errors());
        assert!(result.diagnostics[0].message.contains("undefined entity"));
    }

    #[test]
    fn compile_involving_relationship() {
        let result = compile_source(
            r#"Kael is a character {
    species human
}

the Order is a faction {
    type military
}

the Battle is an event {
    involving [Kael, the Order]
}"#,
        );
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);

        let battle_id = result.world.find_id_by_name("the Battle").unwrap();
        // "involving" creates relationships FROM each participant TO the event
        let rels = result.world.relationships_to(battle_id);
        assert_eq!(rels.len(), 2);
    }

    #[test]
    fn compile_full_world() {
        let result = compile_source(
            r#"world "The Iron Kingdoms" {
    genre "high fantasy"
    setting "A shattered continent rebuilding after the Sundering"
}

the Iron Citadel is a fortress {
    climate arid
    population 45000
    north to the Ashlands

    """
    An ancient fortress carved from a single mountain of iron ore.
    """
}

the Ashlands is a region {
    climate arid
    terrain wasteland
}

Kael Stormborn is a character {
    species human
    occupation knight
    status alive
    traits [brave, stubborn, loyal]
    member of the Order of Dawn
    located at the Iron Citadel
    allied with Elara Nightwhisper
}

Elara Nightwhisper is a character {
    species elf
    occupation mage
    status alive
}

the Order of Dawn is a faction {
    type military_order
    led by Kael Stormborn
    based at the Iron Citadel
    values [honor, duty, sacrifice]
}

the Great Sundering is an event {
    date year -1247, month 3, day 15
    type cataclysm
    involving [Kael Stormborn, the Order of Dawn]

    """
    The day the world broke.
    """
}"#,
        );
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);

        assert_eq!(result.world.meta.name, "The Iron Kingdoms");
        assert_eq!(result.world.entity_count(), 6);
        assert!(result.world.relationship_count() > 0);

        // Verify specific relationships
        let kael_id = result.world.find_id_by_name("Kael Stormborn").unwrap();
        let kael_rels = result.world.relationships_from(kael_id);
        assert!(kael_rels.len() >= 3); // member of, located at, allied with
    }

    // -----------------------------------------------------------------------
    // Large-world stress tests — DSL compilation
    // -----------------------------------------------------------------------

    /// Generate DSL source for N characters, each with properties.
    fn generate_characters(n: usize) -> String {
        let mut src = String::new();
        for i in 0..n {
            src.push_str(&format!(
                "Character{i} is a character {{\n    species human\n    status alive\n}}\n\n"
            ));
        }
        src
    }

    /// Generate DSL source for N locations (mixed subtypes) with exits forming a chain.
    fn generate_location_chain(n: usize) -> String {
        let subtypes = [
            "fortress",
            "city",
            "town",
            "village",
            "region",
            "wilderness",
        ];
        let mut src = String::new();
        for i in 0..n {
            let subtype = subtypes[i % subtypes.len()];
            src.push_str(&format!("Location{i} is a {subtype} {{\n"));
            src.push_str(&format!("    population {}\n", (i + 1) * 100));
            if i + 1 < n {
                src.push_str(&format!("    north to Location{}\n", i + 1));
            }
            src.push_str("}\n\n");
        }
        src
    }

    /// Generate a world with characters that have cross-references.
    fn generate_connected_world(n_chars: usize, n_factions: usize) -> String {
        let mut src = String::new();

        // Factions
        for i in 0..n_factions {
            src.push_str(&format!(
                "Faction{i} is a faction {{\n    type guild\n}}\n\n"
            ));
        }

        // Characters, each a member of a faction and allied with the next character
        for i in 0..n_chars {
            let faction = i % n_factions;
            src.push_str(&format!("Hero{i} is a character {{\n"));
            src.push_str("    species human\n");
            src.push_str(&format!("    member of Faction{faction}\n"));
            if i + 1 < n_chars {
                src.push_str(&format!("    allied with Hero{}\n", i + 1));
            }
            src.push_str("}\n\n");
        }
        src
    }

    #[test]
    fn stress_compile_500_characters() {
        let src = generate_characters(500);
        let result = compile_source(&src);
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);
        assert_eq!(result.world.entity_count(), 500);

        // Spot-check a few entities
        for i in [0, 99, 250, 499] {
            let e = result.world.find_by_name(&format!("Character{i}")).unwrap();
            assert_eq!(e.kind, EntityKind::Character);
            let ch = e.components.character.as_ref().unwrap();
            assert_eq!(ch.species.as_deref(), Some("human"));
        }
    }

    #[test]
    fn stress_compile_200_location_chain() {
        let src = generate_location_chain(200);
        let result = compile_source(&src);
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);
        assert_eq!(result.world.entity_count(), 200);

        // 199 exits (each location connects north to the next, except the last)
        assert_eq!(result.world.relationship_count(), 199);

        // First location has an exit
        let loc0 = result.world.find_id_by_name("Location0").unwrap();
        let rels = result.world.relationships_from(loc0);
        assert!(
            rels.iter().any(|r| r.kind == RelationshipKind::ConnectedTo),
            "Location0 should have a north exit"
        );

        // Last location did not declare an exit, but ConnectedTo is bidirectional,
        // so it still sees the reverse of Location198's north exit.
        let last = result.world.find_id_by_name("Location199").unwrap();
        let rels = result.world.relationships_from(last);
        assert_eq!(
            rels.len(),
            1,
            "last location should only have the reverse of its predecessor's exit"
        );
    }

    #[test]
    fn stress_compile_connected_world() {
        let src = generate_connected_world(200, 10);
        let result = compile_source(&src);
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);

        // 10 factions + 200 characters = 210 entities
        assert_eq!(result.world.entity_count(), 210);

        // Each character has a member-of relationship + alliance (except last)
        // 200 member-of + 199 allied-with = 399
        assert_eq!(result.world.relationship_count(), 399);

        // Verify a faction has members pointing to it
        let f0 = result.world.find_id_by_name("Faction0").unwrap();
        let incoming = result.world.relationships_to(f0);
        // Every 10th character is in Faction0
        assert_eq!(incoming.len(), 20);
    }

    #[test]
    fn stress_compile_mixed_kinds() {
        let mut src = String::from(
            r#"world "Stress Test World" {
    genre "test"
}

"#,
        );

        // 50 of each kind
        for i in 0..50 {
            src.push_str(&format!("Char{i} is a character {{ species human }}\n"));
            src.push_str(&format!(
                "Place{i} is a city {{ population {} }}\n",
                i * 100
            ));
            src.push_str(&format!("Group{i} is a faction {{ type guild }}\n"));
            src.push_str(&format!("Event{i} is an event {{ date year {i} }}\n"));
            src.push_str(&format!("Thing{i} is an item {{ rarity common }}\n"));
            src.push_str(&format!("Lore{i} is lore {{ source \"ancient\" }}\n"));
        }

        let result = compile_source(&src);
        assert!(!result.has_errors(), "errors: {:?}", result.diagnostics);
        assert_eq!(result.world.entity_count(), 300);
        assert_eq!(result.world.meta.name, "Stress Test World");

        let counts = result.world.entity_counts_by_kind();
        assert_eq!(counts[&EntityKind::Character], 50);
        assert_eq!(counts[&EntityKind::Location], 50);
        assert_eq!(counts[&EntityKind::Faction], 50);
        assert_eq!(counts[&EntityKind::Event], 50);
        assert_eq!(counts[&EntityKind::Item], 50);
        assert_eq!(counts[&EntityKind::Lore], 50);
    }
}
