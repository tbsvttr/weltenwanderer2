//! Template registry for narrative text generation.

use std::collections::HashMap;

use ww_core::{Entity, EntityKind, World};

use super::config::{NarratorConfig, NarratorTone, Verbosity};

/// Registry of narrative templates.
#[derive(Debug, Clone)]
pub struct TemplateRegistry {
    config: NarratorConfig,
}

impl TemplateRegistry {
    /// Create a new template registry with the given config.
    pub fn new(config: NarratorConfig) -> Self {
        Self { config }
    }

    /// Describe a location for the player.
    pub fn describe_location(
        &self,
        location: &Entity,
        characters: &[&Entity],
        items: &[&Entity],
        exits: &[String],
    ) -> String {
        let mut output = String::new();

        // Location header
        output.push_str(&self.format_location_header(location));

        // Description
        if !location.description.is_empty() {
            output.push('\n');
            output.push_str(&location.description);
        }

        // Characters present
        if !characters.is_empty() {
            output.push('\n');
            for c in characters {
                output.push_str(&self.format_character_present(c));
                output.push('\n');
            }
        }

        // Items present
        if !items.is_empty() {
            output.push('\n');
            for item in items {
                output.push_str(&self.format_item_present(item));
                output.push('\n');
            }
        }

        // Exits
        if !exits.is_empty() {
            output.push('\n');
            output.push_str(&self.format_exits(exits));
        }

        output
    }

    /// Describe an entity.
    pub fn describe_entity(&self, entity: &Entity, world: &World) -> String {
        let mut output = String::new();

        output.push_str(&format!("**{}**", entity.name));

        if !entity.description.is_empty() {
            output.push('\n');
            output.push_str(&entity.description);
        } else {
            output.push('\n');
            output.push_str(&self.default_entity_description(entity));
        }

        // Show properties based on verbosity
        if self.config.verbosity == Verbosity::Verbose {
            let details = self.entity_details(entity, world);
            if !details.is_empty() {
                output.push('\n');
                output.push_str(&details);
            }
        }

        output
    }

    /// Narrate an arrival at a location.
    pub fn narrate_arrival(&self, location: &Entity) -> String {
        let subject = self.config.player_subject();
        match self.config.tone {
            NarratorTone::Formal => format!("{subject} arrive at {}.", location.name),
            NarratorTone::Casual => format!("{subject} head into {}.", location.name),
            NarratorTone::Dramatic => {
                format!("{subject} set foot upon {}.", location.name)
            }
            NarratorTone::Humorous => {
                format!("{subject} wander into {}. It's a place.", location.name)
            }
        }
    }

    /// Narrate taking an item.
    pub fn narrate_take(&self, item: &Entity) -> String {
        let subject = self.config.player_subject();
        match self.config.tone {
            NarratorTone::Formal => format!("{subject} take {}.", item.name),
            NarratorTone::Casual => format!("{subject} grab {}.", item.name),
            NarratorTone::Dramatic => {
                format!(
                    "{subject} claim {} as {} own.",
                    item.name,
                    self.config.player_possessive()
                )
            }
            NarratorTone::Humorous => {
                format!("{subject} pocket {}. Five-finger discount.", item.name)
            }
        }
    }

    /// Narrate dropping an item.
    pub fn narrate_drop(&self, item: &Entity) -> String {
        let subject = self.config.player_subject();
        match self.config.tone {
            NarratorTone::Formal => format!("{subject} set down {}.", item.name),
            NarratorTone::Casual => format!("{subject} drop {}.", item.name),
            NarratorTone::Dramatic => {
                format!("{subject} relinquish {}.", item.name)
            }
            NarratorTone::Humorous => {
                format!(
                    "{subject} toss {} aside. It wasn't that great anyway.",
                    item.name
                )
            }
        }
    }

    /// Narrate a failed movement.
    pub fn narrate_no_exit(&self, direction: &str) -> String {
        let subject = self.config.player_subject();
        match self.config.tone {
            NarratorTone::Formal => {
                format!("{subject} cannot go {direction} from here.")
            }
            NarratorTone::Casual => {
                format!("There's no way {direction}.")
            }
            NarratorTone::Dramatic => {
                format!(
                    "The path {direction} is barred to {subject}.",
                    subject = self.config.player_object()
                )
            }
            NarratorTone::Humorous => {
                format!("{subject} walk {direction} into a wall. Ouch.")
            }
        }
    }

    /// Format dialogue text from a speaker.
    pub fn format_dialogue(&self, speaker: &str, text: &str) -> String {
        format!("**{speaker}**: {text}")
    }

    /// Format a choice for display.
    pub fn format_choice(&self, index: usize, text: &str) -> String {
        format!("  [{}] {}", index + 1, text)
    }

    fn format_location_header(&self, location: &Entity) -> String {
        format!("**{}**", location.name)
    }

    fn format_character_present(&self, character: &Entity) -> String {
        match self.config.tone {
            NarratorTone::Formal => format!("{} is here.", character.name),
            NarratorTone::Casual => format!("{} is hanging around.", character.name),
            NarratorTone::Dramatic => format!(
                "{} stands before {}.",
                character.name,
                self.config.player_object()
            ),
            NarratorTone::Humorous => format!("{} is here, doing... something.", character.name),
        }
    }

    fn format_item_present(&self, item: &Entity) -> String {
        let subject = self.config.player_subject();
        match self.config.tone {
            NarratorTone::Formal => format!("{subject} see {} here.", item.name),
            NarratorTone::Casual => format!("There's {} lying around.", item.name),
            NarratorTone::Dramatic => format!("{} gleams in the shadows.", item.name),
            NarratorTone::Humorous => format!("{} is just sitting here. Rude.", item.name),
        }
    }

    fn format_exits(&self, exits: &[String]) -> String {
        match self.config.verbosity {
            Verbosity::Terse => format!("[{}]", exits.join(", ")),
            Verbosity::Normal | Verbosity::Verbose => {
                format!("Exits: {}", exits.join(", "))
            }
        }
    }

    fn default_entity_description(&self, entity: &Entity) -> String {
        match entity.kind {
            EntityKind::Character => {
                if let Some(c) = &entity.components.character {
                    let mut parts = Vec::new();
                    if let Some(species) = &c.species {
                        parts.push(format!("A {species}"));
                    }
                    if let Some(occupation) = &c.occupation {
                        parts.push(format!("a {occupation}"));
                    }
                    if parts.is_empty() {
                        "A mysterious figure.".to_string()
                    } else {
                        format!("{}.", parts.join(", "))
                    }
                } else {
                    "A mysterious figure.".to_string()
                }
            }
            EntityKind::Location => "An unremarkable place.".to_string(),
            EntityKind::Item => "An ordinary-looking object.".to_string(),
            _ => "You see nothing special.".to_string(),
        }
    }

    fn entity_details(&self, entity: &Entity, world: &World) -> String {
        let mut details = Vec::new();

        // Show properties
        let props: HashMap<&String, &ww_core::entity::MetadataValue> =
            entity.properties.iter().collect();
        for (key, value) in &props {
            details.push(format!("  {key}: {value}"));
        }

        // Show relationships
        for rel in world.relationships_from(entity.id) {
            if let Some(target) = world.get_entity(rel.target) {
                details.push(format!("  {} {}", rel.kind, target.name));
            }
        }

        details.join("\n")
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new(NarratorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ww_core::Entity;

    #[test]
    fn describe_location_basic() {
        let registry = TemplateRegistry::default();
        let location = {
            let mut e = Entity::new(EntityKind::Location, "the Tavern");
            e.description = "A cozy tavern.".to_string();
            e
        };
        let output = registry.describe_location(&location, &[], &[], &["east".to_string()]);
        assert!(output.contains("Tavern"));
        assert!(output.contains("cozy"));
        assert!(output.contains("east"));
    }

    #[test]
    fn describe_location_with_entities() {
        let registry = TemplateRegistry::default();
        let location = Entity::new(EntityKind::Location, "the Tavern");
        let npc = Entity::new(EntityKind::Character, "Old Tom");
        let item = Entity::new(EntityKind::Item, "pewter mug");
        let output = registry.describe_location(&location, &[&npc], &[&item], &[]);
        assert!(output.contains("Old Tom"));
        assert!(output.contains("pewter mug"));
    }

    #[test]
    fn narrate_arrival_tones() {
        let location = Entity::new(EntityKind::Location, "the Citadel");

        let formal = TemplateRegistry::new(NarratorConfig::new().with_tone(NarratorTone::Formal));
        assert!(formal.narrate_arrival(&location).contains("arrive"));

        let dramatic =
            TemplateRegistry::new(NarratorConfig::new().with_tone(NarratorTone::Dramatic));
        assert!(dramatic.narrate_arrival(&location).contains("set foot"));

        let humorous =
            TemplateRegistry::new(NarratorConfig::new().with_tone(NarratorTone::Humorous));
        assert!(humorous.narrate_arrival(&location).contains("wander"));
    }

    #[test]
    fn third_person_narration() {
        let registry = TemplateRegistry::new(
            NarratorConfig::new()
                .with_perspective(crate::narrator::Perspective::ThirdPerson)
                .with_player_name("Kael"),
        );
        let location = Entity::new(EntityKind::Location, "the Citadel");
        let output = registry.narrate_arrival(&location);
        assert!(output.contains("Kael"));
    }

    #[test]
    fn format_dialogue_and_choices() {
        let registry = TemplateRegistry::default();
        assert_eq!(registry.format_dialogue("Tom", "Hello!"), "**Tom**: Hello!");
        assert_eq!(
            registry.format_choice(0, "Ask about rumors"),
            "  [1] Ask about rumors"
        );
    }
}
