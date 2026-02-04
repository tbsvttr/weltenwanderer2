use std::path::Path;

use colored::Colorize;
use comfy_table::{ContentArrangement, Table};

use ww_core::component::CharacterStatus;
use ww_core::entity::EntityKind;
use ww_simulation::SimEventKind;
use ww_simulation::needs::{NeedKind, NeedsSystem};
use ww_simulation::schedule::ScheduleSystem;
use ww_simulation::spatial::SpatialSystem;
use ww_simulation::{SimConfig, Simulation};

pub fn run(dir: &Path, ticks: u64, seed: u64, speed: f64, verbose: bool) -> Result<(), String> {
    let world = super::compile_dir(dir)?;

    // Collect living characters
    let char_info: Vec<_> = world
        .entities_by_kind(&EntityKind::Character)
        .iter()
        .filter(|e| {
            e.components
                .character
                .as_ref()
                .is_some_and(|c| c.status == CharacterStatus::Alive)
        })
        .map(|e| (e.id, e.name.clone()))
        .collect();

    if char_info.is_empty() {
        println!("  No living characters found. Nothing to simulate.");
        return Ok(());
    }

    // Build and run simulation
    let config = SimConfig::default()
        .with_seed(seed)
        .with_hours_per_tick(speed)
        .with_max_events(500);

    let mut sim = Simulation::new(world, config);
    sim.add_system(NeedsSystem::with_default_config());
    sim.add_system(ScheduleSystem::new());
    sim.add_system(SpatialSystem::new());

    sim.init()
        .map_err(|e| format!("simulation init failed: {e}"))?;
    sim.run(ticks)
        .map_err(|e| format!("simulation error: {e}"))?;

    // Header
    let date = sim.clock().current_date();
    println!(
        "  {} '{}' {}",
        "Simulation".bold(),
        sim.world().meta.name,
        format!("({ticks} ticks, seed={seed}, speed={speed}h/tick)").dimmed()
    );
    println!(
        "  {} characters simulated, {} events logged",
        char_info.len(),
        sim.events().len()
    );
    println!("  In-world date: {date}");
    println!();

    // Events
    if verbose {
        println!("  {}", "Event Log".bold().underline());
        println!();
        for event in sim.events().events() {
            let tick_label = format!("[tick {:>3}]", event.tick).dimmed();
            let desc = colorize_event(&event.kind, &event.description);
            println!("  {tick_label} {desc}");
        }
        if sim.events().is_empty() {
            println!("  {}", "(no events)".dimmed());
        }
        println!();
    } else {
        // Notable events only
        let deaths: Vec<_> = sim
            .events()
            .events()
            .iter()
            .filter(|e| matches!(e.kind, SimEventKind::EntityDied { .. }))
            .collect();
        let criticals: Vec<_> = sim
            .events()
            .events()
            .iter()
            .filter(|e| matches!(e.kind, SimEventKind::NeedCritical { .. }))
            .collect();

        if !deaths.is_empty() || !criticals.is_empty() {
            println!("  {}", "Notable Events".bold().underline());
            for event in &deaths {
                println!("  {}  {}", "DEATH".red().bold(), event.description);
            }
            for event in &criticals {
                println!("  {}   {}", "WARN".yellow().bold(), event.description);
            }
            println!();
        }
    }

    // Character status table
    println!("  {}", "Character Status".bold().underline());
    println!();

    let needs_sys = sim.get_system::<NeedsSystem>();
    let spatial_sys = sim.get_system::<SpatialSystem>();
    let schedule_sys = sim.get_system::<ScheduleSystem>();

    let need_kinds = [
        NeedKind::Hunger,
        NeedKind::Rest,
        NeedKind::Social,
        NeedKind::Safety,
    ];

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        "Character",
        "Hunger",
        "Rest",
        "Social",
        "Safety",
        "Activity",
        "Location",
    ]);

    for (id, name) in &char_info {
        let mut row: Vec<String> = vec![name.clone()];

        // Need bars
        if let Some(ns) = &needs_sys {
            if let Some(state) = ns.get_state(*id) {
                for need in &need_kinds {
                    row.push(format_need_bar(state.get(need).unwrap_or(0.0)));
                }
            } else {
                row.extend(need_kinds.iter().map(|_| "--".to_string()));
            }
        } else {
            row.extend(need_kinds.iter().map(|_| "--".to_string()));
        }

        // Activity
        if let Some(ss) = &schedule_sys {
            row.push(
                ss.current_activity(*id)
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "idle".to_string()),
            );
        } else {
            row.push("--".to_string());
        }

        // Location
        if let Some(sp) = &spatial_sys {
            if let Some(state) = sp.get_state(*id) {
                let loc = sim.world().entity_name(state.current_location);
                if state.is_traveling() {
                    let dest = state
                        .destination
                        .map(|d| sim.world().entity_name(d).to_string())
                        .unwrap_or_default();
                    row.push(format!("{loc} -> {dest}"));
                } else {
                    row.push(loc.to_string());
                }
            } else {
                row.push("--".to_string());
            }
        } else {
            row.push("--".to_string());
        }

        table.add_row(row);
    }

    println!("{table}");
    println!();

    // Death notices
    let world = sim.world();
    let mut any_dead = false;
    for (id, name) in &char_info {
        if let Some(entity) = world.get_entity(*id)
            && let Some(ch) = &entity.components.character
            && ch.status == CharacterStatus::Dead
        {
            if !any_dead {
                println!("  {}", "Status Changes".bold().underline());
                any_dead = true;
            }
            println!("  {} {name}", "DEAD".red().bold());
        }
    }
    if any_dead {
        println!();
    }

    Ok(())
}

fn colorize_event(kind: &SimEventKind, description: &str) -> colored::ColoredString {
    match kind {
        SimEventKind::EntityDied { .. } => description.red().bold(),
        SimEventKind::NeedCritical { .. } => description.yellow(),
        SimEventKind::NeedDepleted { .. } => description.red(),
        SimEventKind::NeedSatisfied { .. } => description.green(),
        SimEventKind::ActivityChanged { .. } => description.cyan(),
        SimEventKind::Departed { .. } | SimEventKind::Arrived { .. } => description.blue(),
        SimEventKind::Custom { .. } => description.normal(),
    }
}

fn format_need_bar(val: f64) -> String {
    let pct = (val * 100.0) as u32;
    let filled = (val * 10.0).round() as usize;
    let empty = 10_usize.saturating_sub(filled);
    let bar = format!("{}{}", "#".repeat(filled), "-".repeat(empty));

    if val <= 0.15 {
        format!("[{}] {:>3}%", bar.red(), pct)
    } else if val <= 0.4 {
        format!("[{}] {:>3}%", bar.yellow(), pct)
    } else {
        format!("[{}] {:>3}%", bar.green(), pct)
    }
}
