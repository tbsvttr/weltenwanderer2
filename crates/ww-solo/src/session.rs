//! Solo TTRPG session management.
//!
//! `SoloSession` embeds a `FictionSession` for world interaction and adds
//! solo-specific commands: oracle queries, scene management, thread/NPC
//! tracking, and journaling.

use chrono::Utc;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

use ww_core::World;
use ww_fiction::FictionSession;
use ww_mechanics::{CharacterSheet, CheckRequest, DicePool, Die, RuleSet};

use crate::chaos::ChaosFactor;
use crate::config::{SoloConfig, SoloWorldConfig};
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
    ruleset: Option<RuleSet>,
    sheet: Option<CharacterSheet>,
    world_config: SoloWorldConfig,
}

impl SoloSession {
    /// Create a new solo session from a compiled world.
    pub fn new(world: World, config: SoloConfig) -> SoloResult<Self> {
        let fiction = FictionSession::new(world)?;
        let rng = StdRng::seed_from_u64(config.seed);
        let chaos = ChaosFactor::new(config.initial_chaos);
        let oracle_config = OracleConfig::from_world(fiction.world());
        let npcs = NpcList::new();

        let world_config = SoloWorldConfig::from_world_meta(&fiction.world().meta.properties);

        // Try to load mechanics (optional — worlds without mechanics still work)
        let ruleset = RuleSet::from_world(fiction.world()).ok();
        let sheet = ruleset.as_ref().and_then(|rs| {
            fiction
                .world()
                .all_entities()
                .filter(|e| e.kind == ww_core::EntityKind::Character)
                .filter(|e| e.properties.keys().any(|k| k.starts_with("mechanics.")))
                .find_map(|e| CharacterSheet::from_entity(e, rs).ok())
        });

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
            ruleset,
            sheet,
            world_config,
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

    /// Get the loaded ruleset, if any.
    pub fn ruleset(&self) -> Option<&RuleSet> {
        self.ruleset.as_ref()
    }

    /// Get the character sheet, if any.
    pub fn sheet(&self) -> Option<&CharacterSheet> {
        self.sheet.as_ref()
    }

    /// Get the world-level solo configuration.
    pub fn world_config(&self) -> &SoloWorldConfig {
        &self.world_config
    }

    /// Get the solo session intro/welcome text.
    ///
    /// Returns custom text from the world's `solo { intro "..." }` block
    /// if defined, otherwise generates a context-aware default based on
    /// whether the world has a ruleset and character sheet.
    pub fn intro(&self) -> String {
        if let Some(ref text) = self.world_config.intro {
            return text.clone();
        }
        self.default_intro()
    }

    fn default_intro(&self) -> String {
        if let Some(rs) = &self.ruleset {
            let mut welcome = format!("**Solo TTRPG Session** — {} system\n\n", rs.name,);
            if let Some(sheet) = &self.sheet {
                welcome.push_str(&format!(
                    "Playing as **{}** ({})\n\n",
                    sheet.name, rs.check_die,
                ));
            }
            welcome.push_str(
                "**Explore** the world, use the **oracle** when\n\
                 you need answers, **roll** when it gets risky.\n\n\
                 **Explore:** look, move, examine, talk\n\
                 **Oracle:** ask <question>, scene <setup>\n\
                 **Mechanics:** check <attr>, roll <dice>, sheet\n\
                 **Journal:** note <text>, journal, status\n\n\
                 Type 'help' for all commands.\n\n",
            );
            welcome
        } else {
            String::from(
                "**Solo TTRPG Session**\n\
                 A Mythic GME-inspired oracle for solo play.\n\n\
                 **Explore** the world, use the **oracle** when\n\
                 you need answers.\n\n\
                 **Explore:** look, move, examine, talk\n\
                 **Oracle:** ask <question>, scene <setup>\n\
                 **Journal:** note <text>, journal, status\n\n\
                 Type 'help' for all commands.\n\n",
            )
        }
    }

    /// Return tab-completion candidates for partial input.
    ///
    /// Given the text the user has typed so far, returns a list of possible
    /// completions as full command strings. Completions expecting further
    /// arguments end with a trailing space.
    pub fn completions(&self, input: &str) -> Vec<String> {
        let trimmed = input.trim_start();

        // Top-level command list (used for empty input and prefix matching)
        let mut commands: Vec<&str> = vec![
            "ask ",
            "reaction ",
            "event",
            "check ",
            "roll ",
            "panic",
            "encounter ",
            "sheet",
            "status",
            "journal",
            "threads",
            "npcs",
            "note ",
            "export ",
            "thread add ",
            "thread close ",
            "thread remove ",
            "npc add ",
            "npc remove ",
            "help",
            "look",
            "examine ",
            "go ",
            "talk ",
        ];

        if self.world_config.enable_chaos {
            commands.push("scene ");
            commands.push("end scene ");
        }

        if trimmed.is_empty() {
            return commands.iter().map(|c| c.to_string()).collect();
        }

        let lower = trimmed.to_lowercase();
        let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        let cmd = parts[0].to_lowercase();
        let rest = parts.get(1).copied().unwrap_or("");

        match cmd.as_str() {
            "ask" if parts.len() > 1 => {
                let rest_lower = rest.to_lowercase();
                let likelihoods = [
                    "impossible",
                    "no way",
                    "very unlikely",
                    "unlikely",
                    "50/50",
                    "somewhat likely",
                    "likely",
                    "very likely",
                    "near sure thing",
                    "a sure thing",
                    "has to be",
                ];
                likelihoods
                    .iter()
                    .filter(|l| l.starts_with(&rest_lower))
                    .map(|l| format!("ask {l} "))
                    .collect()
            }
            "check" if parts.len() > 1 => {
                if let Some(sheet) = &self.sheet {
                    let rest_lower = rest.to_lowercase();
                    sheet
                        .attributes
                        .keys()
                        .filter(|a| a.to_lowercase().starts_with(&rest_lower))
                        .map(|a| format!("check {a}"))
                        .collect()
                } else {
                    vec![]
                }
            }
            "roll" if parts.len() > 1 => {
                let rest_lower = rest.to_lowercase();
                let dice = [
                    "d4", "d6", "d8", "d10", "d12", "d20", "d100", "2d6", "2d10", "2d20",
                ];
                dice.iter()
                    .filter(|d| d.starts_with(&rest_lower))
                    .map(|d| format!("roll {d}"))
                    .collect()
            }
            "encounter" if parts.len() > 1 => {
                let rest_lower = rest.to_lowercase();
                self.fiction
                    .world()
                    .all_entities()
                    .filter(|e| e.name.to_lowercase().starts_with(&rest_lower))
                    .map(|e| format!("encounter {}", e.name))
                    .collect()
            }
            "reaction" if parts.len() > 1 => {
                let rest_lower = rest.to_lowercase();
                self.npcs
                    .list()
                    .iter()
                    .filter(|n| n.name.to_lowercase().starts_with(&rest_lower))
                    .map(|n| format!("reaction {}", n.name))
                    .collect()
            }
            "thread" => {
                let sub_parts: Vec<&str> = rest.splitn(2, ' ').collect();
                let sub = sub_parts[0].to_lowercase();
                let sub_rest = sub_parts.get(1).copied().unwrap_or("");

                if sub.is_empty()
                    || (!["add", "close", "remove"].contains(&sub.as_str())
                        && ["add", "close", "remove"]
                            .iter()
                            .any(|s| s.starts_with(&sub)))
                {
                    ["add", "close", "remove"]
                        .iter()
                        .filter(|s| s.starts_with(&sub))
                        .map(|s| format!("thread {s} "))
                        .collect()
                } else if (sub == "close" || sub == "remove") && sub_parts.len() > 1 {
                    let name_lower = sub_rest.to_lowercase();
                    self.threads
                        .active()
                        .iter()
                        .filter(|t| {
                            name_lower.is_empty() || t.name.to_lowercase().starts_with(&name_lower)
                        })
                        .map(|t| format!("thread {sub} {}", t.name))
                        .collect()
                } else {
                    vec![]
                }
            }
            "npc" => {
                let sub_parts: Vec<&str> = rest.splitn(2, ' ').collect();
                let sub = sub_parts[0].to_lowercase();
                if sub.is_empty() {
                    vec!["npc add ".to_string(), "npc remove ".to_string()]
                } else if sub == "remove" {
                    let name_prefix = sub_parts.get(1).copied().unwrap_or("");
                    if name_prefix.is_empty() {
                        self.npcs
                            .list()
                            .iter()
                            .map(|n| format!("npc remove {}", n.name))
                            .collect()
                    } else {
                        let lower_prefix = name_prefix.to_lowercase();
                        self.npcs
                            .list()
                            .iter()
                            .filter(|n| n.name.to_lowercase().starts_with(&lower_prefix))
                            .map(|n| format!("npc remove {}", n.name))
                            .collect()
                    }
                } else {
                    vec![]
                }
            }
            "export" if parts.len() > 1 => {
                let rest_lower = rest.to_lowercase();
                ["markdown", "text"]
                    .iter()
                    .filter(|f| f.starts_with(&rest_lower))
                    .map(|f| format!("export {f}"))
                    .collect()
            }
            "end" => {
                let rest_lower = rest.to_lowercase();
                if rest_lower.starts_with("scene") {
                    let after = rest.get(5..).map(|s| s.trim_start()).unwrap_or("");
                    if after.is_empty() {
                        vec![
                            "end scene well ".to_string(),
                            "end scene badly ".to_string(),
                        ]
                    } else {
                        let after_lower = after.to_lowercase();
                        ["well ", "badly "]
                            .iter()
                            .filter(|o| o.starts_with(&after_lower))
                            .map(|o| format!("end scene {o}"))
                            .collect()
                    }
                } else {
                    vec!["end scene ".to_string()]
                }
            }
            _ => {
                // Prefix-match top-level commands
                commands
                    .iter()
                    .filter(|c| c.starts_with(&lower))
                    .map(|c| c.to_string())
                    .collect()
            }
        }
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
            "scene" => {
                if !self.world_config.enable_chaos {
                    return Err(SoloError::InvalidChoice(
                        "Scene management is disabled in this world. Use 'ask' for oracle queries."
                            .to_string(),
                    ));
                }
                self.do_scene_start(rest)
            }
            "end" if lower.starts_with("end scene") => {
                if !self.world_config.enable_chaos {
                    return Err(SoloError::InvalidChoice(
                        "Scene management is disabled in this world.".to_string(),
                    ));
                }
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
            "check" => self.do_check(rest),
            "roll" => self.do_roll(rest),
            "panic" => self.do_panic(),
            "encounter" => self.do_encounter(rest),
            "sheet" => self.do_sheet(),
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

        let mut output = if let Some(ref prefix) = self.world_config.oracle_prefix {
            format!(
                "{prefix} {}\n[{}, d100: {} vs {}]",
                result.answer, likelihood, result.roll, result.target,
            )
        } else {
            format!(
                "Oracle: {}\n[{}, d100: {} vs {}]",
                result.answer, likelihood, result.roll, result.target,
            )
        };

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
        let prefix = self
            .world_config
            .reaction_prefix
            .as_deref()
            .unwrap_or("NPC Reaction");
        let output = format!(
            "{prefix} ({npc_name}): {} (roll {})",
            result.reaction, result.roll
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
        let prefix = self
            .world_config
            .event_prefix
            .as_deref()
            .unwrap_or("Random Event:");
        let output = format!("{prefix} {desc}");

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
        let n = self.scene_count;

        let status_text = status.to_string();
        let header = self
            .world_config
            .scene_header
            .as_deref()
            .unwrap_or("--- Scene {n} ---")
            .replace("{n}", &n.to_string());
        let mut output = format!("{header}\nSetup: {setup}\n");

        match &status {
            SceneStatus::Normal => {
                let text = self
                    .world_config
                    .scene_normal
                    .as_deref()
                    .unwrap_or("The scene proceeds as expected.");
                output.push_str(text);
            }
            SceneStatus::Altered => {
                let text = self
                    .world_config
                    .scene_altered
                    .as_deref()
                    .unwrap_or("The scene is ALTERED! Something is different than expected.");
                output.push_str(text);
            }
            SceneStatus::Interrupted(event) => {
                let text = self
                    .world_config
                    .scene_interrupted
                    .as_deref()
                    .unwrap_or("The scene is INTERRUPTED!");
                output.push_str(&format!(
                    "{text}\n  {event}\nSomething completely unexpected happens."
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

        let end_label = self
            .world_config
            .scene_end
            .as_deref()
            .unwrap_or("End of Scene {n}:")
            .replace("{n}", &scene_num.to_string());
        let chaos_label = self
            .world_config
            .chaos_label
            .as_deref()
            .unwrap_or("Chaos factor");
        let output = format!(
            "{end_label} {summary}\n{chaos_label}: {} ({})",
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

    fn do_check(&mut self, rest: &str) -> SoloResult<String> {
        let Some(ruleset) = &self.ruleset else {
            return Err(SoloError::InvalidChoice(
                "no game mechanics defined in this world".to_string(),
            ));
        };
        let Some(sheet) = &self.sheet else {
            return Err(SoloError::InvalidChoice(
                "no character with mechanics found".to_string(),
            ));
        };

        let (attribute, modifier) = parse_check_input(rest)?;

        let request = CheckRequest {
            attribute: Some(attribute.clone()),
            modifier,
            ..CheckRequest::default()
        };

        let result = ww_mechanics::rules::perform_check(ruleset, sheet, &request, &mut self.rng)?;

        let values: Vec<u32> = result.roll.dice.iter().map(|d| d.value).collect();
        let vals_str: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        let dice_desc = format!("{}x{}", result.roll.dice.len(), ruleset.check_die);

        let mut output = format!(
            "Check {attribute}: {dice_desc} = [{}] — {outcome}",
            vals_str.join(", "),
            outcome = result.outcome,
        );

        for effect in &result.effects {
            output.push_str(&format!("\n  {effect}"));
        }

        self.journal.append(JournalEntry::MechanicsCheck {
            attribute,
            dice: dice_desc,
            values,
            outcome: result.outcome.to_string(),
            timestamp: Utc::now(),
        });

        Ok(output)
    }

    fn do_roll(&mut self, rest: &str) -> SoloResult<String> {
        let (count, die) = parse_dice_expression(rest)?;

        let pool = DicePool::new().add(die, count);
        let roll = pool.roll(&mut self.rng);

        let values: Vec<u32> = roll.dice.iter().map(|d| d.value).collect();
        let total: u32 = values.iter().sum();
        let vals_str: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        let expression = format!("{count}{die}");

        let output = format!("Roll {expression}: [{}] = {total}", vals_str.join(", "));

        self.journal.append(JournalEntry::DiceRoll {
            expression,
            values,
            total,
            timestamp: Utc::now(),
        });

        Ok(output)
    }

    fn do_panic(&mut self) -> SoloResult<String> {
        let roll: u32 = self.rng.random_range(1..=20);

        // Get current Stress (or 0 if no sheet/track)
        let stress = self
            .sheet
            .as_ref()
            .and_then(|s| s.tracks.get("Stress"))
            .map(|t| t.current)
            .unwrap_or(0);

        let triggered = roll as i32 <= stress;
        let mut output = format!("PANIC check: d20 -> {roll} (vs Stress {stress})\n");

        if triggered {
            output.push_str(&format!(
                "PANIC! Consult the PANIC chart for effect {roll}."
            ));
        } else {
            output.push_str("No effect — roll exceeds current Stress.");
        }

        // Increment Stress by 1
        if let Some(sheet) = &mut self.sheet
            && let Some(track) = sheet.tracks.get_mut("Stress")
        {
            let old = track.current;
            track.adjust(1);
            output.push_str(&format!("\nStress: {old} -> {}", track.current));
        }

        self.journal.append(JournalEntry::DiceRoll {
            expression: "d20 (PANIC)".to_string(),
            values: vec![roll],
            total: roll,
            timestamp: Utc::now(),
        });

        Ok(output)
    }

    fn do_encounter(&self, name: &str) -> SoloResult<String> {
        if name.is_empty() {
            return Err(SoloError::InvalidChoice(
                "usage: encounter <creature name>".to_string(),
            ));
        }

        let world = self.fiction.world();
        let entity = world
            .find_by_name(name)
            .ok_or_else(|| SoloError::InvalidChoice(format!("creature not found: {name}")))?;

        let mut output = format!("=== {} ===\n", entity.name);

        // Read creature stats from properties
        if let Some(val) = entity.properties.get("type") {
            output.push_str(&format!("Type: {val}\n"));
        }

        let combat = entity.properties.get("combat");
        let instinct = entity.properties.get("instinct");
        let hits = entity.properties.get("hits");

        if combat.is_some() || instinct.is_some() || hits.is_some() {
            let mut stats = Vec::new();
            if let Some(v) = combat {
                stats.push(format!("Combat: {v}"));
            }
            if let Some(v) = instinct {
                stats.push(format!("Instinct: {v}"));
            }
            if let Some(v) = hits {
                stats.push(format!("Hits: {v}"));
            }
            output.push_str(&stats.join(" | "));
            output.push('\n');
        }

        if let Some(val) = entity.properties.get("weapon") {
            output.push_str(&format!("Weapon: {val}\n"));
        }

        // Collect trait-* properties
        let mut traits: Vec<_> = entity
            .properties
            .iter()
            .filter(|(k, _)| k.starts_with("trait-"))
            .collect();
        traits.sort_by_key(|(k, _)| (*k).clone());
        for (key, val) in &traits {
            let trait_name = key.strip_prefix("trait-").unwrap_or(key);
            output.push_str(&format!("- {trait_name}: {val}\n"));
        }

        if !entity.description.is_empty() {
            output.push_str(&format!("\n{}", entity.description));
        }

        Ok(output.trim_end().to_string())
    }

    fn do_sheet(&self) -> SoloResult<String> {
        let Some(ruleset) = &self.ruleset else {
            return Err(SoloError::InvalidChoice(
                "no game mechanics defined in this world".to_string(),
            ));
        };
        let Some(sheet) = &self.sheet else {
            return Err(SoloError::InvalidChoice(
                "no character with mechanics found".to_string(),
            ));
        };

        let mut out = format!(
            "Character: {}\nSystem: {} ({})\n\n",
            sheet.name, ruleset.name, ruleset.check_die
        );

        if !sheet.attributes.is_empty() {
            out.push_str("Attributes:\n");
            let mut attrs: Vec<_> = sheet.attributes.iter().collect();
            attrs.sort_by_key(|(k, _)| (*k).clone());
            for (name, value) in &attrs {
                out.push_str(&format!("  {name}: {value}\n"));
            }
            out.push('\n');
        }

        if !sheet.skills.is_empty() {
            out.push_str("Skills:\n");
            let mut skills: Vec<_> = sheet.skills.iter().collect();
            skills.sort_by_key(|(k, _)| (*k).clone());
            for (name, value) in &skills {
                out.push_str(&format!("  {name}: {value}\n"));
            }
            out.push('\n');
        }

        if !sheet.tracks.is_empty() {
            out.push_str("Tracks:\n");
            let mut tracks: Vec<_> = sheet.tracks.iter().collect();
            tracks.sort_by_key(|(k, _)| (*k).clone());
            for (name, track) in &tracks {
                out.push_str(&format!("  {name}: {}/{}\n", track.current, track.max));
            }
            out.push('\n');
        }

        if !sheet.focuses.is_empty() {
            out.push_str(&format!("Focuses: {}\n", sheet.focuses.join(", ")));
        }

        Ok(out.trim_end().to_string())
    }

    fn do_status(&self) -> SoloResult<String> {
        let mut out = String::new();

        // Only show chaos and scene info if chaos system is enabled
        if self.world_config.enable_chaos {
            let chaos_label = self
                .world_config
                .chaos_label
                .as_deref()
                .unwrap_or("Chaos Factor");
            out.push_str(&format!("{chaos_label}: {}/9\n", self.chaos.value()));

            match &self.current_scene {
                Some(scene) => out.push_str(&format!("Current Scene: #{}\n", scene.number)),
                None => out.push_str("No active scene.\n"),
            }
        }

        out.push_str(&format!(
            "Threads: {} active\n",
            self.threads.active_count()
        ));
        out.push_str(&format!("NPCs: {} tracked\n", self.npcs.count()));
        out.push_str(&format!("Journal: {} entries\n", self.journal.len()));

        if let Some(rs) = &self.ruleset {
            out.push_str(&format!("System: {} ({})\n", rs.name, rs.check_die));
        }
        if let Some(sheet) = &self.sheet {
            out.push_str(&format!("Character: {}", sheet.name));
        }

        Ok(out.trim_end().to_string())
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
            "scene" | "scenes" => {
                if !self.world_config.enable_chaos {
                    return Err(SoloError::InvalidChoice(
                        "Scene management is disabled in this world.".to_string(),
                    ));
                }
                Ok("\
Scene Commands:
  scene <setup>                 Start a new scene (chaos check)
  end scene well <summary>      End scene, chaos decreases
  end scene badly <summary>     End scene, chaos increases"
                    .to_string())
            }
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
            "mechanics" | "check" | "roll" | "sheet" | "panic" | "encounter" => Ok("\
Mechanics Commands:
  check <attribute> [modifier]  Roll a check using world rules
  roll <dice>                   Roll dice (e.g., d100, 2d6, d20)
  panic                         PANIC check (d20 vs Stress, +1 Stress)
  encounter <creature>          Show creature stats from world
  sheet                         Show character attributes and tracks"
                .to_string()),
            _ if self.world_config.help.is_some() => {
                Ok(self.world_config.help.as_ref().unwrap().clone())
            }
            _ => {
                let scene_help = if self.world_config.enable_chaos {
                    "  scene <setup>                 Start a new scene\n  end scene [well|badly] <text> End current scene\n"
                } else {
                    ""
                };
                let help_topics = if self.world_config.enable_chaos {
                    "  help [topic]                  Show help (oracle, scene, mechanics, ...)"
                } else {
                    "  help [topic]                  Show help (oracle, mechanics, ...)"
                };
                Ok(format!(
                    "\
Solo TTRPG Commands:
  ask [likelihood] <question>   Consult the oracle
  reaction <npc>                Roll NPC reaction
  event                         Force a random event
{scene_help}  check <attribute> [modifier]  Roll a mechanics check
  roll <dice>                   Roll dice (d100, 2d6, d20)
  panic                         PANIC check (d20 vs Stress)
  encounter <creature>          Show creature stats
  sheet                         Show character sheet
  thread add|close|remove       Manage plot threads
  threads                       List threads
  npc add|remove                Manage NPCs
  npcs                          List NPCs
  note <text>                   Add journal note
  journal                       Show journal
  export [markdown|text]        Export journal
  status                        Show session status
{help_topics}
  quit                          Exit

World interaction (forwarded to fiction engine):
  look, go, move, take, drop, talk, use, inventory"
                ))
            }
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

/// Parse check input: `<attribute> [modifier]`
fn parse_check_input(input: &str) -> SoloResult<(String, i32)> {
    if input.is_empty() {
        return Err(SoloError::InvalidChoice(
            "usage: check <attribute> [modifier]".to_string(),
        ));
    }
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let attribute = capitalize_first(parts[0]);
    let modifier = parts
        .get(1)
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(0);
    Ok((attribute, modifier))
}

/// Parse a dice expression like "d100", "2d6", "d20".
fn parse_dice_expression(input: &str) -> SoloResult<(u32, Die)> {
    let input = input.trim().to_lowercase();
    if input.is_empty() {
        return Err(SoloError::InvalidChoice(
            "usage: roll <dice> (e.g., d100, 2d6, d20)".to_string(),
        ));
    }

    // Split on 'd' to get count and sides
    let parts: Vec<&str> = input.splitn(2, 'd').collect();
    if parts.len() != 2 || parts[1].is_empty() {
        return Err(SoloError::InvalidChoice(format!(
            "invalid dice expression: {input}"
        )));
    }

    let count = if parts[0].is_empty() {
        1
    } else {
        parts[0]
            .parse::<u32>()
            .map_err(|_| SoloError::InvalidChoice(format!("invalid dice count: {}", parts[0])))?
    };

    if count == 0 {
        return Err(SoloError::InvalidChoice(
            "dice count must be at least 1".to_string(),
        ));
    }

    let die_str = format!("d{}", parts[1]);
    let die = Die::from_str_tag(&die_str)
        .ok_or_else(|| SoloError::InvalidChoice(format!("invalid die type: {die_str}")))?;

    Ok((count, die))
}

/// Capitalize the first letter of a string.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
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
        assert!(output.contains("Oracle:"));
        assert!(output.contains("d100:"));
        assert!(output.contains("Likely"));
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
    fn session_starts_with_empty_npcs() {
        let mut world = World::new(WorldMeta::new("Test World"));
        let tavern = Entity::new(EntityKind::Location, "the Tavern");
        world.add_entity(tavern).unwrap();

        let mut guard = Entity::new(EntityKind::Character, "Guard Captain");
        guard.components.character = Some(ww_core::component::CharacterComponent::default());
        world.add_entity(guard).unwrap();

        let session = SoloSession::new(world, SoloConfig::default()).unwrap();
        assert_eq!(session.npcs().count(), 0);
    }

    // --- Mechanics integration tests ---

    use ww_core::entity::MetadataValue;

    fn mechanics_world() -> World {
        let mut world = World::new(WorldMeta::new("Mechanics World"));
        let tavern = Entity::new(EntityKind::Location, "the Tavern");
        world.add_entity(tavern).unwrap();

        // Ruleset entity
        let mut rules = Entity::new(EntityKind::Custom("ruleset".to_string()), "Game Rules");
        rules.properties.insert(
            "mechanics.system".to_string(),
            MetadataValue::String("mothership".to_string()),
        );
        rules.properties.insert(
            "mechanics.check_die".to_string(),
            MetadataValue::String("d100".to_string()),
        );
        rules
            .properties
            .insert("mechanics.pool_size".to_string(), MetadataValue::Integer(1));
        rules.properties.insert(
            "mechanics.resolution".to_string(),
            MetadataValue::String("roll_under".to_string()),
        );
        rules.properties.insert(
            "mechanics.attributes".to_string(),
            MetadataValue::List(vec![
                MetadataValue::String("Strength".to_string()),
                MetadataValue::String("Speed".to_string()),
                MetadataValue::String("Intellect".to_string()),
                MetadataValue::String("Combat".to_string()),
            ]),
        );
        rules.properties.insert(
            "mechanics.tracks".to_string(),
            MetadataValue::List(vec![
                MetadataValue::String("Health:10:0".to_string()),
                MetadataValue::String("Stress:20:0".to_string()),
            ]),
        );
        world.add_entity(rules).unwrap();

        // Character entity
        let mut character = Entity::new(EntityKind::Character, "Lamplighter");
        character.components.character = Some(ww_core::component::CharacterComponent::default());
        character
            .properties
            .insert("mechanics.strength".to_string(), MetadataValue::Integer(30));
        character
            .properties
            .insert("mechanics.speed".to_string(), MetadataValue::Integer(35));
        character.properties.insert(
            "mechanics.intellect".to_string(),
            MetadataValue::Integer(40),
        );
        character
            .properties
            .insert("mechanics.combat".to_string(), MetadataValue::Integer(25));
        world.add_entity(character).unwrap();

        world
    }

    fn mechanics_session() -> SoloSession {
        SoloSession::new(mechanics_world(), SoloConfig::default()).unwrap()
    }

    #[test]
    fn session_loads_ruleset() {
        let s = mechanics_session();
        assert!(s.ruleset().is_some());
        assert_eq!(s.ruleset().unwrap().name, "mothership");
    }

    #[test]
    fn session_loads_sheet() {
        let s = mechanics_session();
        assert!(s.sheet().is_some());
        assert_eq!(s.sheet().unwrap().name, "Lamplighter");
        assert_eq!(s.sheet().unwrap().attribute("Strength").unwrap(), 30);
    }

    #[test]
    fn session_no_mechanics_still_works() {
        let s = test_session();
        assert!(s.ruleset().is_none());
        assert!(s.sheet().is_none());
    }

    #[test]
    fn check_with_ruleset() {
        let mut s = mechanics_session();
        let output = s.process("check strength").unwrap();
        assert!(output.contains("Check Strength"));
        assert!(output.contains("d100"));
        assert_eq!(s.journal().len(), 1);
    }

    #[test]
    fn check_without_ruleset() {
        let mut s = test_session();
        let result = s.process("check strength");
        assert!(result.is_err());
    }

    #[test]
    fn check_unknown_attribute() {
        let mut s = mechanics_session();
        let result = s.process("check charisma");
        assert!(result.is_err());
    }

    #[test]
    fn check_empty() {
        let mut s = mechanics_session();
        let result = s.process("check");
        assert!(result.is_err());
    }

    #[test]
    fn roll_d100() {
        let mut s = test_session();
        let output = s.process("roll d100").unwrap();
        assert!(output.contains("Roll 1d100"));
        assert_eq!(s.journal().len(), 1);
    }

    #[test]
    fn roll_2d6() {
        let mut s = test_session();
        let output = s.process("roll 2d6").unwrap();
        assert!(output.contains("Roll 2d6"));
    }

    #[test]
    fn roll_invalid() {
        let mut s = test_session();
        assert!(s.process("roll xyz").is_err());
        assert!(s.process("roll").is_err());
    }

    #[test]
    fn sheet_with_ruleset() {
        let mut s = mechanics_session();
        let output = s.process("sheet").unwrap();
        assert!(output.contains("Character: Lamplighter"));
        assert!(output.contains("System: mothership"));
        assert!(output.contains("Strength: 30"));
        assert!(output.contains("Health: 10/10"));
    }

    #[test]
    fn sheet_without_ruleset() {
        let mut s = test_session();
        let result = s.process("sheet");
        assert!(result.is_err());
    }

    #[test]
    fn status_with_mechanics() {
        let s = mechanics_session();
        let status = s.do_status().unwrap();
        assert!(status.contains("System: mothership"));
        assert!(status.contains("Character: Lamplighter"));
    }

    #[test]
    fn help_mechanics_topic() {
        let s = test_session();
        let help = s.do_help("mechanics").unwrap();
        assert!(help.contains("check"));
        assert!(help.contains("roll"));
        assert!(help.contains("sheet"));
    }

    #[test]
    fn help_main_includes_mechanics() {
        let s = test_session();
        let help = s.do_help("").unwrap();
        assert!(help.contains("check"));
        assert!(help.contains("roll"));
        assert!(help.contains("sheet"));
    }

    #[test]
    fn parse_check_input_basic() {
        let (attr, modifier) = parse_check_input("strength").unwrap();
        assert_eq!(attr, "Strength");
        assert_eq!(modifier, 0);
    }

    #[test]
    fn parse_check_input_with_modifier() {
        let (attr, modifier) = parse_check_input("combat -2").unwrap();
        assert_eq!(attr, "Combat");
        assert_eq!(modifier, -2);
    }

    #[test]
    fn parse_dice_expression_d100() {
        let (count, die) = parse_dice_expression("d100").unwrap();
        assert_eq!(count, 1);
        assert_eq!(die, Die::D100);
    }

    #[test]
    fn parse_dice_expression_2d6() {
        let (count, die) = parse_dice_expression("2d6").unwrap();
        assert_eq!(count, 2);
        assert_eq!(die, Die::D6);
    }

    #[test]
    fn parse_dice_expression_invalid() {
        assert!(parse_dice_expression("").is_err());
        assert!(parse_dice_expression("xyz").is_err());
        assert!(parse_dice_expression("0d6").is_err());
    }

    #[test]
    fn journal_mechanics_check_export() {
        let mut s = mechanics_session();
        s.process("check strength").unwrap();
        let md = s.process("export markdown").unwrap();
        assert!(md.contains("**Check**"));
        assert!(md.contains("Strength"));
    }

    #[test]
    fn journal_dice_roll_export() {
        let mut s = test_session();
        s.process("roll 2d6").unwrap();
        let md = s.process("export markdown").unwrap();
        assert!(md.contains("**Roll**"));
        assert!(md.contains("2d6"));
    }

    // --- World config integration tests ---

    fn world_with_solo_config() -> World {
        let mut world = World::new(WorldMeta::new("Configured World"));
        let tavern = Entity::new(EntityKind::Location, "the Tavern");
        world.add_entity(tavern).unwrap();
        world.meta.properties.insert(
            "solo.intro".to_string(),
            MetadataValue::String("Welcome to the dungeon, adventurer.".to_string()),
        );
        world.meta.properties.insert(
            "solo.oracle_prefix".to_string(),
            MetadataValue::String("The spirits whisper...".to_string()),
        );
        world.meta.properties.insert(
            "solo.help".to_string(),
            MetadataValue::String("Use 'go' to move, 'ask' for the oracle.".to_string()),
        );
        world
    }

    #[test]
    fn intro_custom_from_world() {
        let world = world_with_solo_config();
        let session = SoloSession::new(world, SoloConfig::default()).unwrap();
        assert_eq!(session.intro(), "Welcome to the dungeon, adventurer.");
    }

    #[test]
    fn intro_default_with_mechanics() {
        let s = mechanics_session();
        let intro = s.intro();
        assert!(intro.contains("mothership"));
        assert!(intro.contains("Lamplighter"));
    }

    #[test]
    fn intro_default_no_mechanics() {
        let s = test_session();
        let intro = s.intro();
        assert!(intro.contains("Solo TTRPG Session"));
        assert!(intro.contains("Mythic GME"));
    }

    #[test]
    fn oracle_with_prefix() {
        let world = world_with_solo_config();
        let mut s = SoloSession::new(world, SoloConfig::default()).unwrap();
        let output = s.process("ask likely Is there a guard?").unwrap();
        assert!(output.starts_with("The spirits whisper..."));
        assert!(output.contains("d100:"));
    }

    #[test]
    fn oracle_without_prefix() {
        let mut s = test_session();
        let output = s.process("ask likely Is there a guard?").unwrap();
        assert!(output.starts_with("Oracle:"));
    }

    #[test]
    fn help_custom_from_world() {
        let world = world_with_solo_config();
        let s = SoloSession::new(world, SoloConfig::default()).unwrap();
        let help = s.do_help("").unwrap();
        assert_eq!(help, "Use 'go' to move, 'ask' for the oracle.");
    }

    #[test]
    fn help_topic_still_works_with_custom_help() {
        let world = world_with_solo_config();
        let s = SoloSession::new(world, SoloConfig::default()).unwrap();
        let help = s.do_help("oracle").unwrap();
        assert!(help.contains("Likelihood"));
    }

    #[test]
    fn world_config_loaded() {
        let world = world_with_solo_config();
        let s = SoloSession::new(world, SoloConfig::default()).unwrap();
        assert_eq!(
            s.world_config().oracle_prefix.as_deref(),
            Some("The spirits whisper..."),
        );
    }

    // --- Scene/status/event/reaction config tests ---

    fn world_with_full_solo_config() -> World {
        let mut world = World::new(WorldMeta::new("Configured World"));
        let tavern = Entity::new(EntityKind::Location, "the Tavern");
        world.add_entity(tavern).unwrap();
        world.meta.properties.insert(
            "solo.scene_header".to_string(),
            MetadataValue::String("=== Log #{n} ===".to_string()),
        );
        world.meta.properties.insert(
            "solo.scene_normal".to_string(),
            MetadataValue::String("All quiet in the tunnel.".to_string()),
        );
        world.meta.properties.insert(
            "solo.scene_altered".to_string(),
            MetadataValue::String("The walls have shifted.".to_string()),
        );
        world.meta.properties.insert(
            "solo.scene_interrupted".to_string(),
            MetadataValue::String("A tremor runs through the walls.".to_string()),
        );
        world.meta.properties.insert(
            "solo.scene_end".to_string(),
            MetadataValue::String("Log #{n} sealed.".to_string()),
        );
        world.meta.properties.insert(
            "solo.chaos_label".to_string(),
            MetadataValue::String("Pressure".to_string()),
        );
        world.meta.properties.insert(
            "solo.event_prefix".to_string(),
            MetadataValue::String("The tunnel shifts:".to_string()),
        );
        world.meta.properties.insert(
            "solo.reaction_prefix".to_string(),
            MetadataValue::String("Response".to_string()),
        );
        world
    }

    fn full_config_session() -> SoloSession {
        SoloSession::new(world_with_full_solo_config(), SoloConfig::default()).unwrap()
    }

    #[test]
    fn scene_custom_header() {
        let mut s = full_config_session();
        let output = s.process("scene Enter the tunnel").unwrap();
        assert!(output.contains("=== Log #1 ==="));
        assert!(output.contains("Enter the tunnel"));
    }

    #[test]
    fn scene_custom_end() {
        let mut s = full_config_session();
        s.process("scene Enter the tunnel").unwrap();
        let output = s.process("end scene well All clear").unwrap();
        assert!(output.contains("Log #1 sealed."));
        assert!(output.contains("Pressure:"));
    }

    #[test]
    fn chaos_label_custom_in_status() {
        let s = full_config_session();
        let status = s.do_status().unwrap();
        assert!(status.contains("Pressure: 5/9"));
    }

    #[test]
    fn event_prefix_custom() {
        let mut s = full_config_session();
        let output = s.process("event").unwrap();
        assert!(output.starts_with("The tunnel shifts:"));
    }

    #[test]
    fn reaction_prefix_custom() {
        let mut s = full_config_session();
        let output = s.process("reaction Guard").unwrap();
        assert!(output.starts_with("Response (Guard):"));
    }

    // --- Panic tests ---

    /// Mechanics world with Stress track starting at 0 (via character override).
    fn low_stress_world() -> World {
        let mut world = mechanics_world();
        // Override Stress to 0 on the character
        let id = world.find_id_by_name("Lamplighter").unwrap();
        let character = world.get_entity_mut(id).unwrap();
        character
            .properties
            .insert("mechanics.stress".to_string(), MetadataValue::Integer(0));
        world
    }

    #[test]
    fn panic_no_effect() {
        // With Stress at 0, any d20 roll (1-20) > 0 so no panic
        let mut s = SoloSession::new(low_stress_world(), SoloConfig::default()).unwrap();
        let output = s.process("panic").unwrap();
        assert!(output.contains("PANIC check: d20 ->"));
        assert!(output.contains("vs Stress 0"));
        assert!(output.contains("No effect"));
        assert!(output.contains("Stress: 0 -> 1"));
    }

    #[test]
    fn panic_triggered() {
        // Default mechanics world: Stress starts at 20, so d20 (1-20) always <= 20
        let mut s = mechanics_session();
        let output = s.process("panic").unwrap();
        assert!(output.contains("PANIC check: d20 ->"));
        assert!(output.contains("vs Stress 20"));
        assert!(output.contains("PANIC!"));
    }

    #[test]
    fn panic_stress_increments() {
        let mut s = SoloSession::new(low_stress_world(), SoloConfig::default()).unwrap();
        let output1 = s.process("panic").unwrap();
        assert!(output1.contains("Stress: 0 -> 1"));

        let output2 = s.process("panic").unwrap();
        assert!(output2.contains("vs Stress 1"));
        assert!(output2.contains("Stress: 1 -> 2"));
    }

    #[test]
    fn panic_journals_roll() {
        let mut s = mechanics_session();
        let initial_len = s.journal().len();
        s.process("panic").unwrap();
        assert_eq!(s.journal().len(), initial_len + 1);
    }

    // --- Encounter tests ---

    fn creature_world() -> World {
        let mut world = World::new(WorldMeta::new("Creature World"));
        let tavern = Entity::new(EntityKind::Location, "the Tavern");
        world.add_entity(tavern).unwrap();

        let mut crab = Entity::new(
            EntityKind::Custom("creature".to_string()),
            "Spindle-graft Crab",
        );
        crab.properties.insert(
            "type".to_string(),
            MetadataValue::String("wildlife".to_string()),
        );
        crab.properties
            .insert("combat".to_string(), MetadataValue::Integer(40));
        crab.properties
            .insert("instinct".to_string(), MetadataValue::Integer(30));
        crab.properties
            .insert("hits".to_string(), MetadataValue::Integer(2));
        crab.properties.insert(
            "weapon".to_string(),
            MetadataValue::String("Pincer [1d10]".to_string()),
        );
        crab.properties.insert(
            "trait-distress".to_string(),
            MetadataValue::String("will attract other crabs".to_string()),
        );
        world.add_entity(crab).unwrap();
        world
    }

    #[test]
    fn encounter_shows_stats() {
        let s = SoloSession::new(creature_world(), SoloConfig::default()).unwrap();
        let output = s.do_encounter("Spindle-graft Crab").unwrap();
        assert!(output.contains("=== Spindle-graft Crab ==="));
        assert!(output.contains("Type: wildlife"));
        assert!(output.contains("Combat: 40"));
        assert!(output.contains("Instinct: 30"));
        assert!(output.contains("Hits: 2"));
        assert!(output.contains("Weapon: Pincer [1d10]"));
        assert!(output.contains("distress: will attract other crabs"));
    }

    #[test]
    fn encounter_not_found() {
        let s = SoloSession::new(creature_world(), SoloConfig::default()).unwrap();
        let result = s.do_encounter("nonexistent");
        assert!(result.is_err());
    }

    // --- Completion tests ---

    #[test]
    fn completions_empty_returns_commands() {
        let s = test_session();
        let c = s.completions("");
        assert!(c.contains(&"ask ".to_string()));
        assert!(c.contains(&"event".to_string()));
        assert!(c.contains(&"panic".to_string()));
        assert!(c.len() > 10);
    }

    #[test]
    fn completions_partial_command() {
        let s = test_session();
        let c = s.completions("ch");
        assert!(c.contains(&"check ".to_string()));
        assert!(!c.contains(&"ask ".to_string()));
    }

    #[test]
    fn completions_ask_likelihood() {
        let s = test_session();
        let c = s.completions("ask lik");
        assert!(c.contains(&"ask likely ".to_string()));
        assert!(!c.contains(&"ask unlikely ".to_string()));
    }

    #[test]
    fn completions_check_attribute() {
        let s = mechanics_session();
        let c = s.completions("check str");
        assert!(c.contains(&"check Strength".to_string()));
        assert!(!c.contains(&"check Speed".to_string()));
    }

    #[test]
    fn completions_check_empty_lists_all_attributes() {
        let s = mechanics_session();
        let c = s.completions("check ");
        assert!(!c.is_empty());
        assert!(c.contains(&"check Strength".to_string()));
    }

    #[test]
    fn completions_ask_empty_lists_likelihoods() {
        let s = test_session();
        let c = s.completions("ask ");
        assert!(c.contains(&"ask likely ".to_string()));
        assert!(c.contains(&"ask 50/50 ".to_string()));
    }

    #[test]
    fn completions_reaction_npc() {
        let mut s = test_session();
        s.process("npc add Guard Captain").unwrap();
        let c = s.completions("reaction Gu");
        assert!(c.contains(&"reaction Guard Captain".to_string()));
    }

    #[test]
    fn completions_thread_subcommand() {
        let s = test_session();
        let c = s.completions("thread ");
        assert!(c.contains(&"thread add ".to_string()));
        assert!(c.contains(&"thread close ".to_string()));
        assert!(c.contains(&"thread remove ".to_string()));
    }

    #[test]
    fn completions_export_format() {
        let s = test_session();
        let c = s.completions("export m");
        assert!(c.contains(&"export markdown".to_string()));
        assert!(!c.contains(&"export text".to_string()));
    }

    #[test]
    fn completions_end_scene_outcome() {
        let s = test_session();
        let c = s.completions("end scene ");
        assert!(c.contains(&"end scene well ".to_string()));
        assert!(c.contains(&"end scene badly ".to_string()));
    }

    #[test]
    fn completions_encounter_entity() {
        let s = SoloSession::new(creature_world(), SoloConfig::default()).unwrap();
        let c = s.completions("encounter Spin");
        assert!(c.contains(&"encounter Spindle-graft Crab".to_string()));
    }
}
