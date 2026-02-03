use std::path::Path;

use ww_core::timeline::Timeline;

pub fn run(dir: &Path, from: Option<i64>, to: Option<i64>) -> Result<(), String> {
    let world = super::compile_dir(dir)?;

    let timeline = Timeline::from_world(&world).range(from, to);

    if timeline.is_empty() {
        println!("  No events found in the timeline.");
        if from.is_some() || to.is_some() {
            println!("  (try adjusting --from / --to range)");
        }
        return Ok(());
    }

    println!("  Timeline for '{}'", world.meta.name);
    if from.is_some() || to.is_some() {
        let from_str = from.map(|y| y.to_string()).unwrap_or_else(|| "...".into());
        let to_str = to.map(|y| y.to_string()).unwrap_or_else(|| "...".into());
        println!("  Range: {from_str} to {to_str}");
    }
    println!();

    for entry in timeline.entries() {
        let date_str = entry.date.to_string();
        let event_type = entry
            .entity
            .components
            .event
            .as_ref()
            .and_then(|e| e.event_type.as_deref())
            .unwrap_or("");

        let type_suffix = if event_type.is_empty() {
            String::new()
        } else {
            format!(" [{}]", event_type)
        };

        println!("  {:>30}  {}{}", date_str, entry.entity.name, type_suffix);

        if !entry.entity.description.is_empty() {
            let preview = entry.entity.description.lines().next().unwrap_or("");
            let preview = preview.trim();
            if !preview.is_empty() {
                let truncated = if preview.len() > 60 {
                    format!("{}...", &preview[..57])
                } else {
                    preview.to_string()
                };
                println!("  {:>30}  {}", "", truncated);
            }
        }
    }

    println!();
    println!("  {} events", timeline.len());

    Ok(())
}
