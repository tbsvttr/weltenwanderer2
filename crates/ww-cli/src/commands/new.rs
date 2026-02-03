use std::fs;
use std::path::Path;

pub fn run(kind: &str, name: &str, file: Option<&Path>) -> Result<(), String> {
    let stub = generate_stub(kind, name);

    let target = if let Some(path) = file {
        path.to_path_buf()
    } else {
        // Default to <kind>s.ww (e.g. characters.ww, locations.ww)
        let filename = format!("{}s.ww", kind);
        Path::new(&filename).to_path_buf()
    };

    // Append to existing file or create new
    let mut content = if target.exists() {
        let existing = fs::read_to_string(&target)
            .map_err(|e| format!("cannot read {}: {e}", target.display()))?;
        if existing.ends_with('\n') {
            format!("{existing}\n{stub}")
        } else {
            format!("{existing}\n\n{stub}")
        }
    } else {
        stub.clone()
    };

    if !content.ends_with('\n') {
        content.push('\n');
    }

    fs::write(&target, content).map_err(|e| format!("cannot write {}: {e}", target.display()))?;

    println!("  Added {} '{}' to {}", kind, name, target.display());

    Ok(())
}

fn generate_stub(kind: &str, name: &str) -> String {
    let article = match kind.chars().next() {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    };

    match kind {
        "character" => format!(
            r#"{name} is {article} {kind} {{
    species human
    occupation unknown
    status alive
    traits []

    """
    Description of {name}.
    """
}}"#
        ),
        "location" | "fortress" | "city" | "town" | "village" | "region" | "room" => format!(
            r#"{name} is {article} {kind} {{
    climate temperate

    """
    Description of {name}.
    """
}}"#
        ),
        "faction" => format!(
            r#"{name} is {article} {kind} {{
    type organization
    values []

    """
    Description of {name}.
    """
}}"#
        ),
        "event" => format!(
            r#"{name} is {article} {kind} {{
    date year 0
    type occurrence

    """
    Description of {name}.
    """
}}"#
        ),
        "item" => format!(
            r#"{name} is {article} {kind} {{
    type miscellaneous
    rarity common

    """
    Description of {name}.
    """
}}"#
        ),
        "lore" => format!(
            r#"{name} is {kind} {{
    type article
    source "Unknown"

    """
    Description of {name}.
    """
}}"#
        ),
        _ => format!(
            r#"{name} is {article} {kind} {{

    """
    Description of {name}.
    """
}}"#
        ),
    }
}
