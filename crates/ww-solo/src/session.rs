//! Solo TTRPG session management.
//!
//! `SoloSession` embeds a `FictionSession` for world interaction and adds
//! solo-specific commands: oracle queries, scene management, thread/NPC
//! tracking, and journaling.

use chrono::Utc;
use rand::SeedableRng;
use rand::rngs::StdRng;

use ww_core::World;
use ww_fiction::FictionSession;

use crate::chaos::ChaosFactor;
use crate::config::SoloConfig;
use crate::error::{SoloError, SoloResult};
use crate::journal::entry::JournalEntry;
use crate::journal::log::Journal;
use crate::oracle::event::generate_random_event;
use crate::oracle::fate_chart::{Likelihood, consult_oracle};
use crate::oracle::reaction::roll_npc_reaction;
use crate::oracle::tables::OracleConfig;
use crate::scene::{Scene, SceneStatus, check_scene_setup};
use crate::tracker::npcs::NpcList;
use crate::tracker::threads::ThreadList;

/// An interactive solo TTRPG session.
pub struct SoloSession {
    fiction: FictionSession,
    chaos: ChaosFactor,
    oracle_config: OracleConfig,
    current_scene: Option<Scene>,
    scene_count: u32,
    journal: Journal,
    threads: ThreadList,
    npcs: NpcList,
    rng: StdRng,
}

impl SoloSession {
    /// Create a new solo session from a compiled world.
    pub fn new(world: World, config: SoloConfig) -> SoloResult<Self> {
        let fiction = FictionSession::new(world)?;
        let rng = StdRng::seed_from_u64(config.seed);
        let chaos = ChaosFactor::new(config.initial_chaos);
        let oracle_config = OracleConfig::from_world(fiction.world());
        let npcs = NpcList::from_world(fiction.world());

        Ok(Self {
            fiction,
            chaos,
            oracle_config,
            current_scene: None,
            scene_count: 0,
            journal: Journal::new(),
            threads: ThreadList::new(),
            npcs,
            rng,
        })
    }

    /// Get the chaos factor.
    pub fn chaos(&self) -> &ChaosFactor {
        &self.chaos
    }

    /// Get the journal.
    pub fn journal(&self) -> &Journal {
        &self.journal
    }

    /// Get the thread list.
    pub fn threads(&self) -> &ThreadList {
        &self.threads
    }

    /// Get the NPC list.
    pub fn npcs(&self) -> &NpcList {
        &self.npcs
    }

    /// Get the oracle configuration.
    pub fn oracle_config(&self) -> &OracleConfig {
        &self.oracle_config
    }

    /// Get the current scene.
    pub fn current_scene(&self) -> Option<&Scene> {
        self.current_scene.as_ref()
    }

    /// Process a line of user input and return a response.
    pub fn process(&mut self, input: &str) -> SoloResult<String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(String::new());
        }

        let lower = trimmed.to_lowercase();
        let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        let cmd = parts[0].to_lowercase();
        let rest = parts.get(1).map(|s| s.trim()).unwrap_or("");

        match cmd.as_str() {
            "ask" => self.do_oracle(rest),
            "reaction" => self.do_reaction(rest),
            "event" => self.do_event(),
            "scene" => self.do_scene_start(rest),
            "end" if lower.starts_with("end scene") => {
                let after = trimmed
                    .get("end scene".len()..)
                    .map(|s| s.trim())
                    .unwrap_or("");
                self.do_scene_end(after)
            }
            "thread" => self.do_thread(rest),
            "threads" => self.do_thread_list(),
            "npc" => self.do_npc(rest),
            "npcs" => self.do_npc_list(),
            "note" => self.do_note(rest),
            "journal" => self.do_journal_show(),
            "export" => self.do_journal_export(rest),
            "status" => self.do_status(),
            "help" => self.do_help(rest),
            "quit" | "q" => Ok("Goodbye!".to_string()),
            _ => {
                // Forward to fiction session
                self.fiction.process(trimmed).map_err(SoloError::from)
            }
        }
    }

    fn do_oracle(&mut self, rest: &str) -> SoloResult<String> {
        // Parse: ask [likelihood] question?
        // Try to find likelihood as first word, otherwise default to 50/50
        let (likelihood, question) = parse_oracle_input(rest)?;

        let result = consult_oracle(
            likelihood,
            self.chaos.value(),
            &mut self.rng,
            &self.oracle_config,
        );

        let mut output = format!(
            "Oracle ({}, chaos {}): {} (roll {} vs {})",
            likelihood,
            self.chaos.value(),
            result.answer,
            result.roll,
            result.target,
        );

        let random_event_str = if let Some(ref event) = result.random_event {
            let desc = event.to_string();
            output.push_str(&format!("\n  Random Event! {desc}"));
            Some(desc)
        } else {
            None
        };

        self.journal.append(JournalEntry::OracleQuery {
            question: question.to_string(),
            likelihood: likelihood.to_string(),
            chaos: self.chaos.value(),
            result: result.answer.to_string(),
            random_event: random_event_str,
            timestamp: Utc::now(),
        });

        Ok(output)
    }

    fn do_reaction(&mut self, npc_name: &str) -> SoloResult<String> {
        if npc_name.is_empty() {
            return Err(SoloError::InvalidChoice(
                "usage: reaction <npc name>".to_string(),
            ));
        }

        let result = roll_npc_reaction(&mut self.rng);
        let output = format!(
            "NPC Reaction ({}): {} (roll {})",
            npc_name, result.reaction, result.roll
        );

        self.journal.append(JournalEntry::NpcReaction {
            npc_name: npc_name.to_string(),
            reaction: result.reaction.to_string(),
            roll: result.roll,
            timestamp: Utc::now(),
        });

        Ok(output)
    }

    fn do_event(&mut self) -> SoloResult<String> {
        let event = generate_random_event(&mut self.rng, &self.oracle_config);
        let desc = event.to_string();
        let output = format!("Random Event: {desc}");

        self.journal.append(JournalEntry::RandomEvent {
            description: desc,
            timestamp: Utc::now(),
        });

        Ok(output)
    }

    fn do_scene_start(&mut self, setup: &str) -> SoloResult<String> {
        if setup.is_empty() {
            return Err(SoloError::InvalidChoice(
                "usage: scene <setup description>".to_string(),
            ));
        }

        self.scene_count += 1;
        let status = check_scene_setup(self.chaos.value(), &mut self.rng, &self.oracle_config);

        let status_text = status.to_string();
        let mut output = format!("--- Scene {} ---\nSetup: {setup}\n", self.scene_count);

        match &status {
            SceneStatus::Normal => {
                output.push_str("The scene proceeds as expected.");
            }
            SceneStatus::Altered => {
                output.push_str("The scene is ALTERED! Something is different than expected.");
            }
            SceneStatus::Interrupted(event) => {
                output.push_str(&format!(
                    "The scene is INTERRUPTED!\n  {event}\nSomething completely unexpected happens."
                ));
            }
        }

        let scene = Scene {
            number: self.scene_count,
            setup: setup.to_string(),
            status,
            summary: None,
            started_at: Utc::now(),
            ended_at: None,
        };
        self.current_scene = Some(scene);

        self.journal.append(JournalEntry::SceneStart {
            scene_number: self.scene_count,
            setup: setup.to_string(),
            status: status_text,
            timestamp: Utc::now(),
        });

        Ok(output)
    }

    fn do_scene_end(&mut self, rest: &str) -> SoloResult<String> {
        if self.current_scene.is_none() {
            return Err(SoloError::NoActiveScene);
        }

        // Parse: end scene [well/badly] summary
        let (went_well, summary) = parse_scene_end(rest);

        let chaos_adjustment = if went_well { -1 } else { 1 };
        if went_well {
            self.chaos.decrease();
        } else {
            self.chaos.increase();
        }

        let scene_num = self.current_scene.as_ref().unwrap().number;
        self.current_scene = None;

        let output = format!(
            "End of Scene {scene_num}: {summary}\nChaos factor: {} ({})",
            self.chaos.value(),
            if went_well { "-1" } else { "+1" }
        );

        self.journal.append(JournalEntry::SceneEnd {
            scene_number: scene_num,
            summary: summary.to_string(),
            chaos_adjustment,
            timestamp: Utc::now(),
        });

        Ok(output)
    }

    fn do_thread(&mut self, rest: &str) -> SoloResult<String> {
        let parts: Vec<&str> = rest.splitn(2, ' ').collect();
        let sub = parts[0].to_lowercase();
        let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

        match sub.as_str() {
            "add" if !arg.is_empty() => {
                self.threads.add(arg);
                Ok(format!("Thread added: {arg}"))
            }
            "close" if !arg.is_empty() => {
                if self.threads.close(arg) {
                    Ok(format!("Thread closed: {arg}"))
                } else {
                    Ok(format!("Thread not found: {arg}"))
                }
            }
            "remove" if !arg.is_empty() => {
                if self.threads.remove(arg) {
                    Ok(format!("Thread removed: {arg}"))
                } else {
                    Ok(format!("Thread not found: {arg}"))
                }
            }
            _ => Err(SoloError::InvalidChoice(
                "usage: thread add|close|remove <name>".to_string(),
            )),
        }
    }

    fn do_thread_list(&self) -> SoloResult<String> {
        let active = self.threads.active();
        if active.is_empty() {
            return Ok("No active threads.".to_string());
        }
        let mut out = format!("Active threads ({}):\n", active.len());
        for (i, t) in active.iter().enumerate() {
            out.push_str(&format!("  {}. {}\n", i + 1, t.name));
        }
        Ok(out.trim_end().to_string())
    }

    fn do_npc(&mut self, rest: &str) -> SoloResult<String> {
        let parts: Vec<&str> = rest.splitn(2, ' ').collect();
        let sub = parts[0].to_lowercase();
        let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

        match sub.as_str() {
            "add" if !arg.is_empty() => {
                self.npcs.add(arg);
                Ok(format!("NPC added: {arg}"))
            }
            "remove" if !arg.is_empty() => {
                if self.npcs.remove(arg) {
                    Ok(format!("NPC removed: {arg}"))
                } else {
                    Ok(format!("NPC not found: {arg}"))
                }
            }
            _ => Err(SoloError::InvalidChoice(
                "usage: npc add|remove <name>".to_string(),
            )),
        }
    }

    fn do_npc_list(&self) -> SoloResult<String> {
        let list = self.npcs.list();
        if list.is_empty() {
            return Ok("No tracked NPCs.".to_string());
        }
        let mut out = format!("Tracked NPCs ({}):\n", list.len());
        for (i, n) in list.iter().enumerate() {
            out.push_str(&format!("  {}. {}", i + 1, n.name));
            if let Some(notes) = &n.notes {
                out.push_str(&format!(" — {notes}"));
            }
            out.push('\n');
        }
        Ok(out.trim_end().to_string())
    }

    fn do_note(&mut self, text: &str) -> SoloResult<String> {
        if text.is_empty() {
            return Err(SoloError::InvalidChoice("usage: note <text>".to_string()));
        }
        self.journal.append(JournalEntry::Note {
            text: text.to_string(),
            timestamp: Utc::now(),
        });
        Ok("Note recorded.".to_string())
    }

    fn do_journal_show(&self) -> SoloResult<String> {
        if self.journal.is_empty() {
            return Ok("Journal is empty.".to_string());
        }
        // Show last 10 entries as text
        let entries = self.journal.entries();
        let start = entries.len().saturating_sub(10);
        let recent = &entries[start..];

        let mut out = format!(
            "Journal ({} entries, showing last {}):\n\n",
            entries.len(),
            recent.len()
        );
        // Use a mini-journal for formatting
        let mut mini = Journal::new();
        for e in recent {
            mini.append(e.clone());
        }
        out.push_str(&mini.export_text());
        Ok(out.trim_end().to_string())
    }

    fn do_journal_export(&self, format: &str) -> SoloResult<String> {
        match format.to_lowercase().as_str() {
            "markdown" | "md" | "" => Ok(self.journal.export_markdown()),
            "text" | "txt" => Ok(self.journal.export_text()),
            other => Err(SoloError::InvalidChoice(format!(
                "unknown format '{other}', use: markdown, text"
            ))),
        }
    }

    fn do_status(&self) -> SoloResult<String> {
        let mut out = format!("Chaos Factor: {}/9\n", self.chaos.value());

        match &self.current_scene {
            Some(scene) => out.push_str(&format!("Current Scene: #{}\n", scene.number)),
            None => out.push_str("No active scene.\n"),
        }

        out.push_str(&format!(
            "Threads: {} active\n",
            self.threads.active_count()
        ));
        out.push_str(&format!("NPCs: {} tracked\n", self.npcs.count()));
        out.push_str(&format!("Journal: {} entries", self.journal.len()));

        Ok(out)
    }

    fn do_help(&self, topic: &str) -> SoloResult<String> {
        match topic.to_lowercase().as_str() {
            "oracle" | "ask" => Ok("\
Oracle Commands:
  ask [likelihood] <question>   Consult the oracle (yes/no)
  reaction <npc>                Roll NPC reaction (2d10)
  event                         Generate a random event

Likelihood: impossible, no way, very unlikely, unlikely, 50/50,
  somewhat likely, likely, very likely, near sure, sure thing, certain"
                .to_string()),
            "scene" | "scenes" => Ok("\
Scene Commands:
  scene <setup>                 Start a new scene (chaos check)
  end scene well <summary>      End scene, chaos decreases
  end scene badly <summary>     End scene, chaos increases"
                .to_string()),
            "thread" | "threads" => Ok("\
Thread Commands:
  thread add <name>             Add a plot thread
  thread close <name>           Close a thread
  thread remove <name>          Remove a thread
  threads                       List active threads"
                .to_string()),
            "npc" | "npcs" => Ok("\
NPC Commands:
  npc add <name>                Track an NPC
  npc remove <name>             Remove an NPC
  npcs                          List tracked NPCs"
                .to_string()),
            "journal" | "note" => Ok("\
Journal Commands:
  note <text>                   Add a journal note
  journal                       Show recent entries
  export [markdown|text]        Export full journal"
                .to_string()),
            _ => Ok("\
Solo TTRPG Commands:
  ask [likelihood] <question>   Consult the oracle
  reaction <npc>                Roll NPC reaction
  event                         Force a random event
  scene <setup>                 Start a new scene
  end scene [well|badly] <text> End current scene
  thread add|close|remove       Manage plot threads
  threads                       List threads
  npc add|remove                Manage NPCs
  npcs                          List NPCs
  note <text>                   Add journal note
  journal                       Show journal
  export [markdown|text]        Export journal
  status                        Show session status
  help [topic]                  Show help (oracle, scene, thread, npc, journal)
  quit                          Exit

World interaction (forwarded to fiction engine):
  look, go, move, take, drop, talk, use, inventory"
                .to_string()),
        }
    }
}

/// Parse oracle input: `[likelihood] question?`
fn parse_oracle_input(input: &str) -> SoloResult<(Likelihood, &str)> {
    if input.is_empty() {
        return Err(SoloError::InvalidChoice(
            "usage: ask [likelihood] <question>".to_string(),
        ));
    }

    // Try to match the first word(s) as a likelihood
    // Try two-word likelihoods first, then single-word
    let words: Vec<&str> = input.splitn(3, ' ').collect();

    if words.len() >= 3 {
        let two_word = format!("{} {}", words[0], words[1]);
        if let Some(lk) = Likelihood::parse(&two_word) {
            return Ok((lk, words[2]));
        }
    }

    if words.len() >= 2
        && let Some(lk) = Likelihood::parse(words[0])
    {
        return Ok((lk, input[words[0].len()..].trim_start()));
    }

    // Default to 50/50
    Ok((Likelihood::FiftyFifty, input))
}

/// Parse scene end: `[well|badly] summary`
fn parse_scene_end(input: &str) -> (bool, &str) {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    match parts[0].to_lowercase().as_str() {
        "well" | "good" => (true, parts.get(1).unwrap_or(&"")),
        "badly" | "bad" => (false, parts.get(1).unwrap_or(&"")),
        _ => (false, input), // default: badly
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ww_core::{Entity, EntityKind, World, WorldMeta};

    fn test_world() -> World {
        let mut world = World::new(WorldMeta::new("Test World"));
        let tavern = Entity::new(EntityKind::Location, "the Tavern");
        world.add_entity(tavern).unwrap();
        world
    }

    fn test_session() -> SoloSession {
        SoloSession::new(test_world(), SoloConfig::default()).unwrap()
    }

    #[test]
    fn create_session() {
        let s = test_session();
        assert_eq!(s.chaos().value(), 5);
        assert!(s.current_scene().is_none());
        assert!(s.journal().is_empty());
    }

    #[test]
    fn oracle_query() {
        let mut s = test_session();
        let output = s.process("ask likely Is there a guard?").unwrap();
        assert!(output.contains("Oracle"));
        assert!(output.contains("chaos 5"));
        assert_eq!(s.journal().len(), 1);
    }

    #[test]
    fn oracle_default_likelihood() {
        let mut s = test_session();
        let output = s.process("ask Is it raining?").unwrap();
        assert!(output.contains("50/50"));
    }

    #[test]
    fn oracle_two_word_likelihood() {
        let mut s = test_session();
        let output = s.process("ask very likely Is the door open?").unwrap();
        assert!(output.contains("Very Likely"));
    }

    #[test]
    fn npc_reaction() {
        let mut s = test_session();
        let output = s.process("reaction Guard Captain").unwrap();
        assert!(output.contains("NPC Reaction"));
        assert!(output.contains("Guard Captain"));
        assert_eq!(s.journal().len(), 1);
    }

    #[test]
    fn random_event() {
        let mut s = test_session();
        let output = s.process("event").unwrap();
        assert!(output.contains("Random Event"));
        assert_eq!(s.journal().len(), 1);
    }

    #[test]
    fn scene_lifecycle() {
        let mut s = test_session();

        let output = s.process("scene Enter the tavern").unwrap();
        assert!(output.contains("Scene 1"));
        assert!(output.contains("Enter the tavern"));
        assert!(s.current_scene().is_some());

        let output = s.process("end scene well Made an ally").unwrap();
        assert!(output.contains("End of Scene 1"));
        assert!(s.current_scene().is_none());
        assert!(s.chaos().value() <= 5); // decreased or stayed same
    }

    #[test]
    fn scene_end_badly_increases_chaos() {
        let mut s = test_session();
        s.process("scene Enter the dungeon").unwrap();
        s.process("end scene badly Got ambushed").unwrap();
        assert_eq!(s.chaos().value(), 6);
    }

    #[test]
    fn scene_end_well_decreases_chaos() {
        let mut s = test_session();
        s.process("scene Enter the tavern").unwrap();
        s.process("end scene well All went fine").unwrap();
        assert_eq!(s.chaos().value(), 4);
    }

    #[test]
    fn scene_end_without_active() {
        let mut s = test_session();
        let result = s.process("end scene well done");
        assert!(result.is_err());
    }

    #[test]
    fn thread_management() {
        let mut s = test_session();
        assert_eq!(
            s.process("thread add Find the artifact").unwrap(),
            "Thread added: Find the artifact"
        );
        assert_eq!(s.threads().active_count(), 1);

        let list = s.process("threads").unwrap();
        assert!(list.contains("Find the artifact"));

        s.process("thread close Find the artifact").unwrap();
        assert_eq!(s.threads().active_count(), 0);
    }

    #[test]
    fn npc_management() {
        let mut s = test_session();
        s.process("npc add Guard Captain").unwrap();
        assert_eq!(s.npcs().count(), 1);

        let list = s.process("npcs").unwrap();
        assert!(list.contains("Guard Captain"));

        s.process("npc remove Guard Captain").unwrap();
        assert_eq!(s.npcs().count(), 0);
    }

    #[test]
    fn note_and_journal() {
        let mut s = test_session();
        s.process("note The guard seemed nervous").unwrap();
        assert_eq!(s.journal().len(), 1);

        let journal = s.process("journal").unwrap();
        assert!(journal.contains("The guard seemed nervous"));
    }

    #[test]
    fn journal_export() {
        let mut s = test_session();
        s.process("note Test entry").unwrap();

        let md = s.process("export markdown").unwrap();
        assert!(md.contains("# Solo Session Journal"));

        let txt = s.process("export text").unwrap();
        assert!(txt.contains("Solo Session Journal"));
    }

    #[test]
    fn status() {
        let mut s = test_session();
        s.process("thread add Quest").unwrap();
        s.process("npc add Guard").unwrap();
        s.process("note Hello").unwrap();

        let status = s.process("status").unwrap();
        assert!(status.contains("Chaos Factor: 5/9"));
        assert!(status.contains("Threads: 1 active"));
        assert!(status.contains("NPCs: 1 tracked"));
        assert!(status.contains("Journal: 1 entries"));
    }

    #[test]
    fn help_commands() {
        let s = test_session();
        let help = s.do_help("").unwrap();
        assert!(help.contains("Solo TTRPG Commands"));

        let help = s.do_help("oracle").unwrap();
        assert!(help.contains("Likelihood"));
    }

    #[test]
    fn forward_to_fiction() {
        let mut s = test_session();
        let output = s.process("look").unwrap();
        // Should get a location description from the fiction engine
        assert!(!output.is_empty());
    }

    #[test]
    fn quit() {
        let mut s = test_session();
        let output = s.process("quit").unwrap();
        assert_eq!(output, "Goodbye!");
    }

    #[test]
    fn empty_input() {
        let mut s = test_session();
        let output = s.process("").unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn parse_oracle_input_likely() {
        let (lk, q) = parse_oracle_input("likely Is there a guard?").unwrap();
        assert_eq!(lk, Likelihood::Likely);
        assert_eq!(q, "Is there a guard?");
    }

    #[test]
    fn parse_oracle_input_default() {
        let (lk, q) = parse_oracle_input("Is it raining?").unwrap();
        assert_eq!(lk, Likelihood::FiftyFifty);
        assert_eq!(q, "Is it raining?");
    }

    #[test]
    fn parse_oracle_input_two_word() {
        let (lk, q) = parse_oracle_input("very unlikely Did the dragon wake?").unwrap();
        assert_eq!(lk, Likelihood::VeryUnlikely);
        assert_eq!(q, "Did the dragon wake?");
    }

    #[test]
    fn parse_scene_end_well() {
        let (well, summary) = parse_scene_end("well Made a friend");
        assert!(well);
        assert_eq!(summary, "Made a friend");
    }

    #[test]
    fn parse_scene_end_badly() {
        let (well, summary) = parse_scene_end("badly Got captured");
        assert!(!well);
        assert_eq!(summary, "Got captured");
    }

    #[test]
    fn parse_scene_end_default() {
        let (well, summary) = parse_scene_end("something happened");
        assert!(!well);
        assert_eq!(summary, "something happened");
    }

    #[test]
    fn session_auto_populates_npcs() {
        let mut world = World::new(WorldMeta::new("Test World"));
        let tavern = Entity::new(EntityKind::Location, "the Tavern");
        world.add_entity(tavern).unwrap();

        let mut guard = Entity::new(EntityKind::Character, "Guard Captain");
        guard.components.character = Some(ww_core::component::CharacterComponent::default());
        world.add_entity(guard).unwrap();

        let session = SoloSession::new(world, SoloConfig::default()).unwrap();
        assert_eq!(session.npcs().count(), 1);
        assert_eq!(session.npcs().list()[0].name, "Guard Captain");
    }
}
