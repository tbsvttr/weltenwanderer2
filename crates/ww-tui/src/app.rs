//! Top-level application state managing tabs and shared world data.

use ww_core::World;

use crate::tabs::explorer::ExplorerTab;
use crate::tabs::graph::GraphTab;
use crate::tabs::timeline::TimelineTab;
use crate::tabs::{InputMode, Tab, TabId};

/// Main application state for the unified TUI.
pub struct TuiApp {
    /// The loaded world.
    pub world: World,
    /// Currently active tab.
    pub active_tab: TabId,
    /// Whether to show the global help popup.
    pub show_help: bool,
    /// Whether the app should quit.
    pub should_quit: bool,

    // Tab instances
    /// Explorer tab (always initialized).
    pub explorer: ExplorerTab,
    /// Graph tab (always initialized).
    pub graph: GraphTab,
    /// Timeline tab (always initialized).
    pub timeline: TimelineTab,
    /// Play tab (lazily initialized).
    pub play: Option<crate::tabs::play::PlayTab>,
    /// Solo tab (lazily initialized).
    pub solo: Option<crate::tabs::solo::SoloTab>,
    /// Sheet tab (always initialized).
    pub sheet: crate::tabs::sheet::SheetTab,
    /// Dice tab (always initialized).
    pub dice: crate::tabs::dice::DiceTab,

    // Solo config
    /// RNG seed for solo/dice.
    pub seed: u64,
    /// Initial chaos factor.
    pub chaos: u32,
}

impl TuiApp {
    /// Create a new app from a compiled world.
    pub fn new(world: World, start_tab: TabId, seed: u64, chaos: u32) -> Self {
        let explorer = ExplorerTab::new(world.clone());
        let graph = GraphTab::new(world.clone());
        let timeline = TimelineTab::new(world.clone());
        let sheet = crate::tabs::sheet::SheetTab::new(world.clone());
        let dice = crate::tabs::dice::DiceTab::new(seed);

        Self {
            world,
            active_tab: start_tab,
            show_help: false,
            should_quit: false,
            explorer,
            graph,
            timeline,
            play: None,
            solo: None,
            sheet,
            dice,
            seed,
            chaos,
        }
    }

    /// Get the input mode of the currently active tab.
    pub fn active_input_mode(&self) -> InputMode {
        self.active_tab_ref().input_mode()
    }

    /// Get a reference to the active tab.
    pub fn active_tab_ref(&self) -> &dyn Tab {
        match self.active_tab {
            TabId::Explorer => &self.explorer,
            TabId::Graph => &self.graph,
            TabId::Timeline => &self.timeline,
            TabId::Play => self
                .play
                .as_ref()
                .map(|t| t as &dyn Tab)
                .unwrap_or(&self.explorer),
            TabId::Solo => self
                .solo
                .as_ref()
                .map(|t| t as &dyn Tab)
                .unwrap_or(&self.explorer),
            TabId::Sheet => &self.sheet,
            TabId::Dice => &self.dice,
        }
    }

    /// Get a mutable reference to the active tab.
    pub fn active_tab_mut(&mut self) -> &mut dyn Tab {
        match self.active_tab {
            TabId::Explorer => &mut self.explorer,
            TabId::Graph => &mut self.graph,
            TabId::Timeline => &mut self.timeline,
            TabId::Play => {
                if self.play.is_none() {
                    self.play = Some(crate::tabs::play::PlayTab::new(self.world.clone()));
                }
                self.play.as_mut().unwrap()
            }
            TabId::Solo => {
                if self.solo.is_none() {
                    let config = ww_solo::SoloConfig::default()
                        .with_seed(self.seed)
                        .with_chaos(self.chaos);
                    match crate::tabs::solo::SoloTab::new(self.world.clone(), config) {
                        Ok(tab) => self.solo = Some(tab),
                        Err(_) => return &mut self.explorer,
                    }
                }
                self.solo.as_mut().unwrap()
            }
            TabId::Sheet => &mut self.sheet,
            TabId::Dice => &mut self.dice,
        }
    }

    /// Switch to a tab by ID, lazily initializing if needed.
    pub fn switch_tab(&mut self, tab: TabId) {
        self.active_tab = tab;
        // Ensure lazy tabs are initialized by calling active_tab_mut
        let _ = self.active_tab_mut();
    }
}
