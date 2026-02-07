//! Command parsing for player input.

/// Direction for movement commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// North.
    North,
    /// South.
    South,
    /// East.
    East,
    /// West.
    West,
    /// Up.
    Up,
    /// Down.
    Down,
    /// Northeast.
    Northeast,
    /// Northwest.
    Northwest,
    /// Southeast.
    Southeast,
    /// Southwest.
    Southwest,
}

impl Direction {
    /// Parse a direction from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "n" | "north" => Some(Self::North),
            "s" | "south" => Some(Self::South),
            "e" | "east" => Some(Self::East),
            "w" | "west" => Some(Self::West),
            "u" | "up" => Some(Self::Up),
            "d" | "down" => Some(Self::Down),
            "ne" | "northeast" => Some(Self::Northeast),
            "nw" | "northwest" => Some(Self::Northwest),
            "se" | "southeast" => Some(Self::Southeast),
            "sw" | "southwest" => Some(Self::Southwest),
            _ => None,
        }
    }

    /// Get the display name for this direction.
    pub fn name(&self) -> &'static str {
        match self {
            Self::North => "north",
            Self::South => "south",
            Self::East => "east",
            Self::West => "west",
            Self::Up => "up",
            Self::Down => "down",
            Self::Northeast => "northeast",
            Self::Northwest => "northwest",
            Self::Southeast => "southeast",
            Self::Southwest => "southwest",
        }
    }
}

/// A parsed player command.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Move in a cardinal direction.
    Move {
        /// The direction to move.
        direction: Direction,
    },
    /// Go to a named location.
    Go {
        /// The target location name.
        target: String,
    },
    /// Look at the current location or a specific target.
    Look {
        /// Optional target to examine.
        target: Option<String>,
    },
    /// Take an item.
    Take {
        /// The item name.
        item: String,
    },
    /// Drop an item.
    Drop {
        /// The item name.
        item: String,
    },
    /// Talk to a character.
    Talk {
        /// The character name.
        character: String,
        /// Optional topic to discuss.
        topic: Option<String>,
    },
    /// Use an item, optionally on a target.
    Use {
        /// The item name.
        item: String,
        /// Optional target to use item on.
        target: Option<String>,
    },
    /// List inventory.
    Inventory,
    /// Show help.
    Help {
        /// Optional help topic.
        topic: Option<String>,
    },
    /// Quit the game.
    Quit,
    /// Unknown command.
    Unknown {
        /// The original input.
        input: String,
    },
}

/// Verb synonyms for command parsing.
const MOVE_VERBS: &[&str] = &["go", "move", "walk", "head", "travel"];
const LOOK_VERBS: &[&str] = &["look", "l", "examine", "ex", "x", "describe", "inspect"];
const TAKE_VERBS: &[&str] = &["take", "get", "pick", "grab"];
const DROP_VERBS: &[&str] = &["drop", "put", "leave", "discard"];
const TALK_VERBS: &[&str] = &["talk", "speak", "ask", "chat", "converse"];
const USE_VERBS: &[&str] = &["use", "apply", "activate"];
const INVENTORY_VERBS: &[&str] = &["inventory", "inv", "i", "items"];
const HELP_VERBS: &[&str] = &["help", "h", "?", "commands"];
const QUIT_VERBS: &[&str] = &["quit", "q", "exit", "bye"];

/// Parse a player input string into a command.
pub fn parse_command(input: &str) -> Command {
    let input = input.trim();
    if input.is_empty() {
        return Command::Look { target: None };
    }

    let words: Vec<&str> = input.split_whitespace().collect();
    let verb = words[0].to_lowercase();
    let rest = words.get(1..).unwrap_or(&[]);

    // Check for bare direction
    if let Some(dir) = Direction::parse(&verb) {
        return Command::Move { direction: dir };
    }

    // Check verb categories
    if MOVE_VERBS.contains(&verb.as_str()) {
        return parse_move(rest);
    }
    if LOOK_VERBS.contains(&verb.as_str()) {
        return parse_look(rest);
    }
    if TAKE_VERBS.contains(&verb.as_str()) {
        return parse_take(rest);
    }
    if DROP_VERBS.contains(&verb.as_str()) {
        return parse_drop(rest);
    }
    if TALK_VERBS.contains(&verb.as_str()) {
        return parse_talk(rest);
    }
    if USE_VERBS.contains(&verb.as_str()) {
        return parse_use(rest);
    }
    if INVENTORY_VERBS.contains(&verb.as_str()) {
        return Command::Inventory;
    }
    if HELP_VERBS.contains(&verb.as_str()) {
        return parse_help(rest);
    }
    if QUIT_VERBS.contains(&verb.as_str()) {
        return Command::Quit;
    }

    Command::Unknown {
        input: input.to_string(),
    }
}

fn parse_move(rest: &[&str]) -> Command {
    if rest.is_empty() {
        return Command::Look { target: None };
    }

    // Check if first word is a direction
    if let Some(dir) = Direction::parse(rest[0]) {
        return Command::Move { direction: dir };
    }

    // Otherwise treat as named location
    Command::Go {
        target: rest.join(" "),
    }
}

fn parse_look(rest: &[&str]) -> Command {
    if rest.is_empty() {
        return Command::Look { target: None };
    }

    // Skip "at" if present
    let target_words = if rest[0].eq_ignore_ascii_case("at") {
        &rest[1..]
    } else {
        rest
    };

    if target_words.is_empty() {
        Command::Look { target: None }
    } else {
        Command::Look {
            target: Some(target_words.join(" ")),
        }
    }
}

fn parse_take(rest: &[&str]) -> Command {
    if rest.is_empty() {
        return Command::Unknown {
            input: "take what?".to_string(),
        };
    }

    // Skip "up" if present (pick up)
    let item_words = if rest[0].eq_ignore_ascii_case("up") {
        &rest[1..]
    } else {
        rest
    };

    if item_words.is_empty() {
        Command::Unknown {
            input: "take what?".to_string(),
        }
    } else {
        Command::Take {
            item: item_words.join(" "),
        }
    }
}

fn parse_drop(rest: &[&str]) -> Command {
    if rest.is_empty() {
        return Command::Unknown {
            input: "drop what?".to_string(),
        };
    }

    Command::Drop {
        item: rest.join(" "),
    }
}

fn parse_talk(rest: &[&str]) -> Command {
    if rest.is_empty() {
        return Command::Unknown {
            input: "talk to whom?".to_string(),
        };
    }

    // Skip "to" or "with" if present
    let remaining = if rest[0].eq_ignore_ascii_case("to") || rest[0].eq_ignore_ascii_case("with") {
        &rest[1..]
    } else {
        rest
    };

    if remaining.is_empty() {
        return Command::Unknown {
            input: "talk to whom?".to_string(),
        };
    }

    // Check for "about" to split character and topic
    if let Some(about_pos) = remaining
        .iter()
        .position(|w| w.eq_ignore_ascii_case("about"))
    {
        let character = remaining[..about_pos].join(" ");
        let topic = remaining[about_pos + 1..].join(" ");
        Command::Talk {
            character,
            topic: if topic.is_empty() { None } else { Some(topic) },
        }
    } else {
        Command::Talk {
            character: remaining.join(" "),
            topic: None,
        }
    }
}

fn parse_use(rest: &[&str]) -> Command {
    if rest.is_empty() {
        return Command::Unknown {
            input: "use what?".to_string(),
        };
    }

    // Check for "on" or "with" to split item and target
    if let Some(split_pos) = rest
        .iter()
        .position(|w| w.eq_ignore_ascii_case("on") || w.eq_ignore_ascii_case("with"))
    {
        let item = rest[..split_pos].join(" ");
        let target = rest[split_pos + 1..].join(" ");
        Command::Use {
            item,
            target: if target.is_empty() {
                None
            } else {
                Some(target)
            },
        }
    } else {
        Command::Use {
            item: rest.join(" "),
            target: None,
        }
    }
}

fn parse_help(rest: &[&str]) -> Command {
    if rest.is_empty() {
        Command::Help { topic: None }
    } else {
        Command::Help {
            topic: Some(rest.join(" ")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bare_direction() {
        assert_eq!(
            parse_command("north"),
            Command::Move {
                direction: Direction::North
            }
        );
        assert_eq!(
            parse_command("n"),
            Command::Move {
                direction: Direction::North
            }
        );
        assert_eq!(
            parse_command("sw"),
            Command::Move {
                direction: Direction::Southwest
            }
        );
    }

    #[test]
    fn parse_go_direction() {
        assert_eq!(
            parse_command("go north"),
            Command::Move {
                direction: Direction::North
            }
        );
        assert_eq!(
            parse_command("walk east"),
            Command::Move {
                direction: Direction::East
            }
        );
    }

    #[test]
    fn parse_go_location() {
        assert_eq!(
            parse_command("go the Iron Citadel"),
            Command::Go {
                target: "the Iron Citadel".to_string()
            }
        );
    }

    #[test]
    fn parse_look() {
        assert_eq!(parse_command("look"), Command::Look { target: None });
        assert_eq!(parse_command("l"), Command::Look { target: None });
        assert_eq!(
            parse_command("look sword"),
            Command::Look {
                target: Some("sword".to_string())
            }
        );
        assert_eq!(
            parse_command("examine at the chest"),
            Command::Look {
                target: Some("the chest".to_string())
            }
        );
    }

    #[test]
    fn parse_take() {
        assert_eq!(
            parse_command("take sword"),
            Command::Take {
                item: "sword".to_string()
            }
        );
        assert_eq!(
            parse_command("pick up the golden key"),
            Command::Take {
                item: "the golden key".to_string()
            }
        );
    }

    #[test]
    fn parse_drop() {
        assert_eq!(
            parse_command("drop sword"),
            Command::Drop {
                item: "sword".to_string()
            }
        );
    }

    #[test]
    fn parse_talk() {
        assert_eq!(
            parse_command("talk to Kael"),
            Command::Talk {
                character: "Kael".to_string(),
                topic: None
            }
        );
        assert_eq!(
            parse_command("ask Kael Stormborn about the Ashlands"),
            Command::Talk {
                character: "Kael Stormborn".to_string(),
                topic: Some("the Ashlands".to_string())
            }
        );
    }

    #[test]
    fn parse_use() {
        assert_eq!(
            parse_command("use key"),
            Command::Use {
                item: "key".to_string(),
                target: None
            }
        );
        assert_eq!(
            parse_command("use key on door"),
            Command::Use {
                item: "key".to_string(),
                target: Some("door".to_string())
            }
        );
    }

    #[test]
    fn parse_inventory() {
        assert_eq!(parse_command("inventory"), Command::Inventory);
        assert_eq!(parse_command("i"), Command::Inventory);
    }

    #[test]
    fn parse_help() {
        assert_eq!(parse_command("help"), Command::Help { topic: None });
        assert_eq!(
            parse_command("help movement"),
            Command::Help {
                topic: Some("movement".to_string())
            }
        );
    }

    #[test]
    fn parse_quit() {
        assert_eq!(parse_command("quit"), Command::Quit);
        assert_eq!(parse_command("q"), Command::Quit);
    }

    #[test]
    fn parse_unknown() {
        assert_eq!(
            parse_command("dance wildly"),
            Command::Unknown {
                input: "dance wildly".to_string()
            }
        );
    }

    #[test]
    fn empty_input_is_look() {
        assert_eq!(parse_command(""), Command::Look { target: None });
        assert_eq!(parse_command("   "), Command::Look { target: None });
    }
}
