//! Interactive fiction session management.

use crate::error::{FictionError, FictionResult};
use crate::parser::{Command, Direction, parse_command, resolve_entity};
use crate::player::PlayerState;
use ww_core::{EntityId, EntityKind, RelationshipKind, World};

/// An interactive fiction session.
pub struct FictionSession {
    /// The world being explored.
    world: World,
    /// The player's current state.
    player: PlayerState,
}

impl FictionSession {
    /// Create a new fiction session.
    ///
    /// The player will be placed at the first location found,
    /// or returns an error if no locations exist.
    pub fn new(world: World) -> FictionResult<Self> {
        // Find a starting location
        let locations = world.entities_by_kind(&EntityKind::Location);
        let start_location = locations
            .first()
            .map(|e| e.id)
            .ok_or_else(|| FictionError::LocationNotFound("no locations in world".to_string()))?;

        // Create a player entity ID (not added to world, just for tracking)
        let player_id = EntityId::new();

        let player = PlayerState::new(player_id, start_location);

        Ok(Self { world, player })
    }

    /// Create a session with the player at a specific location.
    pub fn at_location(world: World, location_name: &str) -> FictionResult<Self> {
        let location = world
            .find_by_name(location_name)
            .ok_or_else(|| FictionError::LocationNotFound(location_name.to_string()))?;

        let player_id = EntityId::new();
        let player = PlayerState::new(player_id, location.id);

        Ok(Self { world, player })
    }

    /// Get the current world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get the player state.
    pub fn player(&self) -> &PlayerState {
        &self.player
    }

    /// Get a mutable reference to the player state.
    pub fn player_mut(&mut self) -> &mut PlayerState {
        &mut self.player
    }

    /// Process a player command and return a response.
    pub fn process(&mut self, input: &str) -> FictionResult<String> {
        let command = parse_command(input);
        self.execute(command)
    }

    /// Execute a parsed command.
    pub fn execute(&mut self, command: Command) -> FictionResult<String> {
        match command {
            Command::Move { direction } => self.do_move(direction),
            Command::Go { target } => self.do_go(&target),
            Command::Look { target } => self.do_look(target.as_deref()),
            Command::Take { item } => self.do_take(&item),
            Command::Drop { item } => self.do_drop(&item),
            Command::Talk { character, topic } => self.do_talk(&character, topic.as_deref()),
            Command::Use { item, target } => self.do_use(&item, target.as_deref()),
            Command::Inventory => self.do_inventory(),
            Command::Help { topic } => self.do_help(topic.as_deref()),
            Command::Quit => Ok("Goodbye!".to_string()),
            Command::Unknown { input } => Err(FictionError::UnknownCommand(input)),
        }
    }

    fn do_move(&mut self, direction: Direction) -> FictionResult<String> {
        let current = self.player.location;

        // Find exit in that direction
        let exit = self
            .world
            .relationships_from(current)
            .into_iter()
            .find(|r| {
                r.kind == RelationshipKind::ConnectedTo
                    && r.label
                        .as_ref()
                        .is_some_and(|l| l.to_lowercase() == direction.name())
            })
            .map(|r| r.target);

        if let Some(destination) = exit {
            self.player.location = destination;
            self.do_look(None)
        } else {
            Ok(format!("You can't go {} from here.", direction.name()))
        }
    }

    fn do_go(&mut self, target: &str) -> FictionResult<String> {
        // First check if it's a direction
        if let Some(dir) = Direction::parse(target) {
            return self.do_move(dir);
        }

        // Otherwise try to find the location
        let destination = resolve_entity(&self.world, target)
            .ok_or_else(|| FictionError::LocationNotFound(target.to_string()))?;

        let entity = self.world.get_entity(destination);
        if entity.is_none() || entity.unwrap().kind != EntityKind::Location {
            return Err(FictionError::LocationNotFound(target.to_string()));
        }

        self.player.location = destination;
        self.do_look(None)
    }

    fn do_look(&self, target: Option<&str>) -> FictionResult<String> {
        if let Some(target_name) = target {
            // Look at a specific entity
            let entity_id = resolve_entity(&self.world, target_name)
                .ok_or_else(|| FictionError::EntityNotFound(target_name.to_string()))?;
            let entity = self.world.get_entity(entity_id).unwrap();
            Ok(self.describe_entity(entity))
        } else {
            // Look at current location
            let location = self
                .world
                .get_entity(self.player.location)
                .ok_or_else(|| FictionError::LocationNotFound("current location".to_string()))?;
            Ok(self.describe_location(location))
        }
    }

    fn describe_location(&self, location: &ww_core::Entity) -> String {
        let mut output = format!("**{}**\n", location.name);

        if !location.description.is_empty() {
            output.push_str(&location.description);
            output.push('\n');
        }

        // List characters here
        let characters: Vec<_> = self
            .world
            .all_entities()
            .filter(|e| {
                e.kind == EntityKind::Character
                    && self.world.relationships_from(e.id).iter().any(|r| {
                        r.target == location.id
                            && matches!(
                                r.kind,
                                RelationshipKind::LocatedAt
                                    | RelationshipKind::BasedAt
                                    | RelationshipKind::ContainedIn
                            )
                    })
            })
            .collect();

        if !characters.is_empty() {
            output.push('\n');
            for c in characters {
                output.push_str(&format!("{} is here.\n", c.name));
            }
        }

        // List items here
        let items: Vec<_> = self
            .world
            .all_entities()
            .filter(|e| {
                e.kind == EntityKind::Item
                    && self.world.relationships_from(e.id).iter().any(|r| {
                        r.target == location.id
                            && matches!(
                                r.kind,
                                RelationshipKind::LocatedAt | RelationshipKind::ContainedIn
                            )
                    })
            })
            .collect();

        if !items.is_empty() {
            output.push('\n');
            for item in items {
                output.push_str(&format!("You see {} here.\n", item.name));
            }
        }

        // List exits
        let exits: Vec<_> = self
            .world
            .relationships_from(location.id)
            .into_iter()
            .filter(|r| r.kind == RelationshipKind::ConnectedTo)
            .filter_map(|r| r.label.clone())
            .collect();

        if !exits.is_empty() {
            output.push_str(&format!("\nExits: {}", exits.join(", ")));
        }

        output
    }

    fn describe_entity(&self, entity: &ww_core::Entity) -> String {
        let mut output = format!("**{}**\n", entity.name);

        if !entity.description.is_empty() {
            output.push_str(&entity.description);
        } else {
            output.push_str("You see nothing special.");
        }

        output
    }

    fn do_take(&mut self, item_name: &str) -> FictionResult<String> {
        let item_id = resolve_entity(&self.world, item_name)
            .ok_or_else(|| FictionError::EntityNotFound(item_name.to_string()))?;

        let entity = self.world.get_entity(item_id);
        if entity.is_none() || entity.unwrap().kind != EntityKind::Item {
            return Err(FictionError::CannotTake(item_name.to_string()));
        }

        let name = entity.unwrap().name.clone();

        // Check if item is at current location
        let at_location = self.world.relationships_from(item_id).iter().any(|r| {
            r.target == self.player.location
                && matches!(
                    r.kind,
                    RelationshipKind::LocatedAt | RelationshipKind::ContainedIn
                )
        });

        if !at_location {
            return Err(FictionError::EntityNotFound(item_name.to_string()));
        }

        self.player.add_item(item_id);
        Ok(format!("You take {}.", name))
    }

    fn do_drop(&mut self, item_name: &str) -> FictionResult<String> {
        let item_id = resolve_entity(&self.world, item_name)
            .ok_or_else(|| FictionError::ItemNotInInventory(item_name.to_string()))?;

        if !self.player.has_item(item_id) {
            return Err(FictionError::ItemNotInInventory(item_name.to_string()));
        }

        let name = self
            .world
            .get_entity(item_id)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| item_name.to_string());

        self.player.remove_item(item_id);
        Ok(format!("You drop {}.", name))
    }

    fn do_talk(&self, character_name: &str, topic: Option<&str>) -> FictionResult<String> {
        let char_id = resolve_entity(&self.world, character_name)
            .ok_or_else(|| FictionError::EntityNotFound(character_name.to_string()))?;

        let entity = self.world.get_entity(char_id);
        if entity.is_none() || entity.unwrap().kind != EntityKind::Character {
            return Err(FictionError::EntityNotFound(character_name.to_string()));
        }

        let character = entity.unwrap();

        // Check for DSL-defined dialogues
        if let Some(fiction) = &character.components.fiction {
            // Find matching dialogue by topic or use first available
            let dialogue = if let Some(t) = topic {
                fiction.dialogues.iter().find(|d| d.id == t)
            } else {
                fiction.dialogues.first()
            };

            if let Some(dlg) = dialogue {
                let mut output = format!("**{}**: {}", character.name, dlg.text);

                if !dlg.choices.is_empty() {
                    output.push('\n');
                    for (i, choice) in dlg.choices.iter().enumerate() {
                        output.push_str(&format!("\n  [{}] {}", i + 1, choice.text));
                    }
                }

                return Ok(output);
            }
        }

        Ok(format!("{} has nothing to say.", character.name))
    }

    fn do_use(&self, item_name: &str, target: Option<&str>) -> FictionResult<String> {
        let item_id = resolve_entity(&self.world, item_name)
            .ok_or_else(|| FictionError::ItemNotInInventory(item_name.to_string()))?;

        if !self.player.has_item(item_id) {
            return Err(FictionError::ItemNotInInventory(item_name.to_string()));
        }

        let item = self.world.get_entity(item_id).unwrap();

        if let Some(target_name) = target {
            Ok(format!(
                "You use {} on {}. Nothing happens.",
                item.name, target_name
            ))
        } else {
            Ok(format!("You use {}. Nothing happens.", item.name))
        }
    }

    fn do_inventory(&self) -> FictionResult<String> {
        if self.player.inventory.is_empty() {
            return Ok("You are carrying nothing.".to_string());
        }

        let mut output = "You are carrying:\n".to_string();
        for item_id in &self.player.inventory {
            if let Some(item) = self.world.get_entity(*item_id) {
                output.push_str(&format!("  - {}\n", item.name));
            }
        }

        Ok(output)
    }

    fn do_help(&self, topic: Option<&str>) -> FictionResult<String> {
        if let Some(t) = topic {
            match t.to_lowercase().as_str() {
                "movement" | "move" | "go" => Ok("**Movement**\n\
                    Use cardinal directions: north, south, east, west, up, down\n\
                    Or abbreviations: n, s, e, w, u, d\n\
                    You can also: go <location name>"
                    .to_string()),
                "look" | "examine" => Ok("**Looking**\n\
                    look - describe current location\n\
                    look <target> - examine something specific"
                    .to_string()),
                "inventory" | "items" => Ok("**Inventory**\n\
                    take <item> - pick up an item\n\
                    drop <item> - drop an item\n\
                    inventory (or i) - list what you're carrying"
                    .to_string()),
                "talk" | "dialogue" => Ok("**Talking**\n\
                    talk to <character> - start a conversation\n\
                    ask <character> about <topic> - ask about something specific"
                    .to_string()),
                _ => Ok(format!("No help available for '{}'.", t)),
            }
        } else {
            Ok("**Commands**\n\
                Movement: north, south, east, west, up, down (or n, s, e, w, u, d)\n\
                go <location> - travel to a named location\n\
                look [target] - examine surroundings or something specific\n\
                take <item> - pick up an item\n\
                drop <item> - drop an item\n\
                inventory (or i) - list what you're carrying\n\
                talk to <character> - start a conversation\n\
                use <item> [on <target>] - use an item\n\
                help [topic] - show help\n\
                quit - exit the game\n\n\
                Type 'help <topic>' for more details."
                .to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ww_core::{Entity, Relationship, WorldMeta};

    fn test_world() -> World {
        let mut world = World::new(WorldMeta::new("Test World"));

        // Create locations
        let mut tavern = Entity::new(EntityKind::Location, "the Rusty Tankard");
        tavern.description = "A cozy tavern with a roaring fire.".to_string();
        let street = Entity::new(EntityKind::Location, "Market Street");

        let tavern_id = world.add_entity(tavern).unwrap();
        let street_id = world.add_entity(street).unwrap();

        // Connect them
        world
            .add_relationship(
                Relationship::new(tavern_id, RelationshipKind::ConnectedTo, street_id)
                    .with_label("east"),
            )
            .unwrap();
        world
            .add_relationship(
                Relationship::new(street_id, RelationshipKind::ConnectedTo, tavern_id)
                    .with_label("west"),
            )
            .unwrap();

        // Add a character
        let mut barkeep = Entity::new(EntityKind::Character, "Old Tom");
        barkeep.description = "A grizzled old man with kind eyes.".to_string();
        let barkeep_id = world.add_entity(barkeep).unwrap();
        world
            .add_relationship(Relationship::new(
                barkeep_id,
                RelationshipKind::LocatedAt,
                tavern_id,
            ))
            .unwrap();

        // Add an item
        let mug = Entity::new(EntityKind::Item, "pewter mug");
        let mug_id = world.add_entity(mug).unwrap();
        world
            .add_relationship(Relationship::new(
                mug_id,
                RelationshipKind::LocatedAt,
                tavern_id,
            ))
            .unwrap();

        world
    }

    #[test]
    fn create_session() {
        let world = test_world();
        let session = FictionSession::new(world).unwrap();
        assert!(!session.world().meta.name.is_empty());
    }

    #[test]
    fn look_at_location() {
        let world = test_world();
        let session = FictionSession::at_location(world, "the Rusty Tankard").unwrap();
        let output = session.do_look(None).unwrap();

        assert!(output.contains("Rusty Tankard"));
        assert!(output.contains("cozy tavern"));
        assert!(output.contains("Old Tom"));
        assert!(output.contains("pewter mug"));
        assert!(output.contains("east"));
    }

    #[test]
    fn move_direction() {
        let world = test_world();
        let mut session = FictionSession::at_location(world, "the Rusty Tankard").unwrap();

        let output = session.do_move(Direction::East).unwrap();
        assert!(output.contains("Market Street"));
    }

    #[test]
    fn move_invalid_direction() {
        let world = test_world();
        let mut session = FictionSession::at_location(world, "the Rusty Tankard").unwrap();

        let output = session.do_move(Direction::North).unwrap();
        assert!(output.contains("can't go north"));
    }

    #[test]
    fn take_item() {
        let world = test_world();
        let mut session = FictionSession::at_location(world, "the Rusty Tankard").unwrap();

        let output = session.do_take("pewter mug").unwrap();
        assert!(output.contains("take"));
        assert!(session.player().inventory.len() == 1);
    }

    #[test]
    fn inventory_empty() {
        let world = test_world();
        let session = FictionSession::at_location(world, "the Rusty Tankard").unwrap();
        let output = session.do_inventory().unwrap();
        assert!(output.contains("nothing"));
    }

    #[test]
    fn inventory_with_items() {
        let world = test_world();
        let mut session = FictionSession::at_location(world, "the Rusty Tankard").unwrap();
        session.do_take("pewter mug").unwrap();

        let output = session.do_inventory().unwrap();
        assert!(output.contains("pewter mug"));
    }

    #[test]
    fn process_command() {
        let world = test_world();
        let mut session = FictionSession::at_location(world, "the Rusty Tankard").unwrap();

        let output = session.process("look").unwrap();
        assert!(output.contains("Rusty Tankard"));

        let output = session.process("e").unwrap();
        assert!(output.contains("Market Street"));
    }

    #[test]
    fn help_command() {
        let world = test_world();
        let session = FictionSession::at_location(world, "the Rusty Tankard").unwrap();

        let output = session.do_help(None).unwrap();
        assert!(output.contains("Commands"));

        let output = session.do_help(Some("movement")).unwrap();
        assert!(output.contains("Movement"));
    }

    #[test]
    fn talk_with_dialogue() {
        use ww_core::component::{ChoiceData, DialogueData, FictionComponent};

        let mut world = test_world();

        // Add dialogue data to Old Tom
        let tom_id = world.find_id_by_name("Old Tom").unwrap();
        let tom = world.get_entity_mut(tom_id).unwrap();
        tom.components.fiction = Some(FictionComponent {
            dialogues: vec![DialogueData {
                id: "greeting".to_string(),
                text: "Welcome to the Rusty Tankard!".to_string(),
                conditions: vec![],
                choices: vec![
                    ChoiceData {
                        text: "Any rumors?".to_string(),
                        response: "Strange lights in the Ashlands.".to_string(),
                        effects: vec![],
                        conditions: vec![],
                        goto: None,
                    },
                    ChoiceData {
                        text: "Just passing through.".to_string(),
                        response: "Safe travels.".to_string(),
                        effects: vec![],
                        conditions: vec![],
                        goto: None,
                    },
                ],
            }],
        });

        let session = FictionSession::at_location(world, "the Rusty Tankard").unwrap();
        let output = session.do_talk("Old Tom", None).unwrap();

        assert!(output.contains("Old Tom"));
        assert!(output.contains("Welcome to the Rusty Tankard!"));
        assert!(output.contains("[1] Any rumors?"));
        assert!(output.contains("[2] Just passing through."));
    }

    #[test]
    fn talk_no_dialogue_fallback() {
        let world = test_world();
        let session = FictionSession::at_location(world, "the Rusty Tankard").unwrap();

        let output = session.do_talk("Old Tom", None).unwrap();
        assert!(output.contains("nothing to say"));
    }
}
