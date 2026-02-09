//! Journal storage and export.

use serde::{Deserialize, Serialize};

use super::entry::JournalEntry;

/// A chronological log of session events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Journal {
    entries: Vec<JournalEntry>,
}

impl Journal {
    /// Create an empty journal.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an entry to the journal.
    pub fn append(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    /// Get all entries.
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the journal is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Export the journal as markdown.
    pub fn export_markdown(&self) -> String {
        let mut out = String::from("# Solo Session Journal\n\n");
        for entry in &self.entries {
            match entry {
                JournalEntry::OracleQuery {
                    question,
                    likelihood,
                    result,
                    random_event,
                    ..
                } => {
                    out.push_str(&format!("**Oracle** ({likelihood}): {question}\n"));
                    out.push_str(&format!("  **Answer**: {result}\n"));
                    if let Some(event) = random_event {
                        out.push_str(&format!("  *Random Event*: {event}\n"));
                    }
                    out.push('\n');
                }
                JournalEntry::SceneStart {
                    scene_number,
                    setup,
                    status,
                    ..
                } => {
                    out.push_str(&format!("## Scene {scene_number}\n\n"));
                    out.push_str(&format!("**Setup**: {setup}\n"));
                    out.push_str(&format!("**Status**: {status}\n\n"));
                }
                JournalEntry::SceneEnd {
                    scene_number,
                    summary,
                    chaos_adjustment,
                    ..
                } => {
                    out.push_str(&format!("*End of Scene {scene_number}*: {summary}\n"));
                    let adj = match chaos_adjustment {
                        1.. => "+1",
                        0 => "unchanged",
                        _ => "-1",
                    };
                    out.push_str(&format!("Chaos: {adj}\n\n"));
                }
                JournalEntry::NarrativeBeat { text, .. } => {
                    out.push_str(&format!("{text}\n\n"));
                }
                JournalEntry::Note { text, .. } => {
                    out.push_str(&format!("> {text}\n\n"));
                }
                JournalEntry::NpcReaction {
                    npc_name,
                    reaction,
                    roll,
                    ..
                } => {
                    out.push_str(&format!(
                        "**NPC Reaction** ({npc_name}): {reaction} (roll: {roll})\n\n"
                    ));
                }
                JournalEntry::RandomEvent { description, .. } => {
                    out.push_str(&format!("*Random Event*: {description}\n\n"));
                }
                JournalEntry::MechanicsCheck {
                    attribute,
                    dice,
                    values,
                    outcome,
                    ..
                } => {
                    let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                    out.push_str(&format!(
                        "**Check** ({attribute}): {dice} = [{}] — **{outcome}**\n\n",
                        vals.join(", ")
                    ));
                }
                JournalEntry::DiceRoll {
                    expression,
                    values,
                    total,
                    ..
                } => {
                    let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                    out.push_str(&format!(
                        "**Roll** {expression}: [{}] = {total}\n\n",
                        vals.join(", ")
                    ));
                }
            }
        }
        out
    }

    /// Export the journal as plain text.
    pub fn export_text(&self) -> String {
        let mut out = String::from("Solo Session Journal\n====================\n\n");
        for entry in &self.entries {
            match entry {
                JournalEntry::OracleQuery {
                    question,
                    likelihood,
                    result,
                    random_event,
                    ..
                } => {
                    out.push_str(&format!("Oracle ({likelihood}): {question}\n"));
                    out.push_str(&format!("  Answer: {result}\n"));
                    if let Some(event) = random_event {
                        out.push_str(&format!("  Random Event: {event}\n"));
                    }
                    out.push('\n');
                }
                JournalEntry::SceneStart {
                    scene_number,
                    setup,
                    status,
                    ..
                } => {
                    out.push_str(&format!("--- Scene {scene_number} ---\n"));
                    out.push_str(&format!("Setup: {setup}\n"));
                    out.push_str(&format!("Status: {status}\n\n"));
                }
                JournalEntry::SceneEnd {
                    scene_number,
                    summary,
                    chaos_adjustment,
                    ..
                } => {
                    out.push_str(&format!("End of Scene {scene_number}: {summary}\n"));
                    let adj = match chaos_adjustment {
                        1.. => "+1",
                        0 => "unchanged",
                        _ => "-1",
                    };
                    out.push_str(&format!("Chaos: {adj}\n\n"));
                }
                JournalEntry::NarrativeBeat { text, .. } => {
                    out.push_str(&format!("{text}\n\n"));
                }
                JournalEntry::Note { text, .. } => {
                    out.push_str(&format!("Note: {text}\n\n"));
                }
                JournalEntry::NpcReaction {
                    npc_name,
                    reaction,
                    roll,
                    ..
                } => {
                    out.push_str(&format!(
                        "NPC Reaction ({npc_name}): {reaction} (roll: {roll})\n\n"
                    ));
                }
                JournalEntry::RandomEvent { description, .. } => {
                    out.push_str(&format!("Random Event: {description}\n\n"));
                }
                JournalEntry::MechanicsCheck {
                    attribute,
                    dice,
                    values,
                    outcome,
                    ..
                } => {
                    let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                    out.push_str(&format!(
                        "Check ({attribute}): {dice} = [{}] — {outcome}\n\n",
                        vals.join(", ")
                    ));
                }
                JournalEntry::DiceRoll {
                    expression,
                    values,
                    total,
                    ..
                } => {
                    let vals: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                    out.push_str(&format!(
                        "Roll {expression}: [{}] = {total}\n\n",
                        vals.join(", ")
                    ));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn empty_journal() {
        let j = Journal::new();
        assert!(j.is_empty());
        assert_eq!(j.len(), 0);
    }

    #[test]
    fn append_and_query() {
        let mut j = Journal::new();
        j.append(JournalEntry::Note {
            text: "Test note".to_string(),
            timestamp: Utc::now(),
        });
        assert_eq!(j.len(), 1);
        assert!(!j.is_empty());
    }

    #[test]
    fn export_markdown_oracle() {
        let mut j = Journal::new();
        j.append(JournalEntry::OracleQuery {
            question: "Is there a guard?".to_string(),
            likelihood: "Likely".to_string(),
            chaos: 5,
            result: "Yes".to_string(),
            random_event: None,
            timestamp: Utc::now(),
        });
        let md = j.export_markdown();
        assert!(md.contains("**Oracle** (Likely): Is there a guard?"));
        assert!(md.contains("**Answer**: Yes"));
    }

    #[test]
    fn export_markdown_scene() {
        let mut j = Journal::new();
        j.append(JournalEntry::SceneStart {
            scene_number: 1,
            setup: "Enter the tavern".to_string(),
            status: "Normal".to_string(),
            timestamp: Utc::now(),
        });
        j.append(JournalEntry::SceneEnd {
            scene_number: 1,
            summary: "Made an ally".to_string(),
            chaos_adjustment: -1,
            timestamp: Utc::now(),
        });
        let md = j.export_markdown();
        assert!(md.contains("## Scene 1"));
        assert!(md.contains("Enter the tavern"));
        assert!(md.contains("Chaos: -1"));
    }

    #[test]
    fn export_text_note() {
        let mut j = Journal::new();
        j.append(JournalEntry::Note {
            text: "Remember the key".to_string(),
            timestamp: Utc::now(),
        });
        let txt = j.export_text();
        assert!(txt.contains("Note: Remember the key"));
    }

    #[test]
    fn export_markdown_with_random_event() {
        let mut j = Journal::new();
        j.append(JournalEntry::OracleQuery {
            question: "Is the door locked?".to_string(),
            likelihood: "50/50".to_string(),
            chaos: 5,
            result: "No".to_string(),
            random_event: Some("NPC Action: Betray + Allies".to_string()),
            timestamp: Utc::now(),
        });
        let md = j.export_markdown();
        assert!(md.contains("*Random Event*: NPC Action: Betray + Allies"));
    }

    #[test]
    fn export_markdown_npc_reaction() {
        let mut j = Journal::new();
        j.append(JournalEntry::NpcReaction {
            npc_name: "Guard Captain".to_string(),
            reaction: "Cautious".to_string(),
            roll: 7,
            timestamp: Utc::now(),
        });
        let md = j.export_markdown();
        assert!(md.contains("Guard Captain"));
        assert!(md.contains("Cautious"));
    }

    #[test]
    fn journal_serde_roundtrip() {
        let mut j = Journal::new();
        j.append(JournalEntry::Note {
            text: "test".to_string(),
            timestamp: Utc::now(),
        });
        let json = serde_json::to_string(&j).unwrap();
        let j2: Journal = serde_json::from_str(&json).unwrap();
        assert_eq!(j2.len(), 1);
    }

    #[test]
    fn export_markdown_mechanics_check() {
        let mut j = Journal::new();
        j.append(JournalEntry::MechanicsCheck {
            attribute: "Strength".to_string(),
            dice: "1xd100".to_string(),
            values: vec![42],
            outcome: "Success (margin 8)".to_string(),
            timestamp: Utc::now(),
        });
        let md = j.export_markdown();
        assert!(md.contains("**Check** (Strength)"));
        assert!(md.contains("[42]"));
        assert!(md.contains("**Success (margin 8)**"));
    }

    #[test]
    fn export_text_mechanics_check() {
        let mut j = Journal::new();
        j.append(JournalEntry::MechanicsCheck {
            attribute: "Speed".to_string(),
            dice: "1xd100".to_string(),
            values: vec![71],
            outcome: "Failure".to_string(),
            timestamp: Utc::now(),
        });
        let txt = j.export_text();
        assert!(txt.contains("Check (Speed)"));
        assert!(txt.contains("Failure"));
    }

    #[test]
    fn export_markdown_dice_roll() {
        let mut j = Journal::new();
        j.append(JournalEntry::DiceRoll {
            expression: "2d6".to_string(),
            values: vec![3, 5],
            total: 8,
            timestamp: Utc::now(),
        });
        let md = j.export_markdown();
        assert!(md.contains("**Roll** 2d6"));
        assert!(md.contains("[3, 5]"));
        assert!(md.contains("= 8"));
    }

    #[test]
    fn export_text_dice_roll() {
        let mut j = Journal::new();
        j.append(JournalEntry::DiceRoll {
            expression: "d20".to_string(),
            values: vec![17],
            total: 17,
            timestamp: Utc::now(),
        });
        let txt = j.export_text();
        assert!(txt.contains("Roll d20"));
        assert!(txt.contains("= 17"));
    }

    #[test]
    fn serde_roundtrip_mechanics_entries() {
        let mut j = Journal::new();
        j.append(JournalEntry::MechanicsCheck {
            attribute: "Combat".to_string(),
            dice: "1xd100".to_string(),
            values: vec![55],
            outcome: "Critical Failure".to_string(),
            timestamp: Utc::now(),
        });
        j.append(JournalEntry::DiceRoll {
            expression: "3d6".to_string(),
            values: vec![2, 4, 6],
            total: 12,
            timestamp: Utc::now(),
        });
        let json = serde_json::to_string(&j).unwrap();
        let j2: Journal = serde_json::from_str(&json).unwrap();
        assert_eq!(j2.len(), 2);
    }
}
