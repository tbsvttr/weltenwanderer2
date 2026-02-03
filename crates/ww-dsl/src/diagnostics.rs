use ariadne::{Color, Label, Report, ReportKind, Source};
use std::fmt;

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A diagnostic message with source location.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub span: std::ops::Range<usize>,
    pub message: String,
    pub label: Option<String>,
}

impl Diagnostic {
    pub fn error(span: std::ops::Range<usize>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            span,
            message: message.into(),
            label: None,
        }
    }

    pub fn warning(span: std::ops::Range<usize>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            span,
            message: message.into(),
            label: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{prefix}: {}", self.message)
    }
}

/// Render diagnostics using ariadne for pretty terminal output.
pub fn render_diagnostics(source: &str, filename: &str, diagnostics: &[Diagnostic]) -> String {
    let mut output = Vec::new();

    for diag in diagnostics {
        let kind = match diag.severity {
            Severity::Error => ReportKind::Error,
            Severity::Warning => ReportKind::Warning,
        };
        let color = match diag.severity {
            Severity::Error => Color::Red,
            Severity::Warning => Color::Yellow,
        };

        let span = (filename, diag.span.clone());
        let mut report = Report::build(kind, span).with_message(&diag.message);

        let label_text = diag.label.as_deref().unwrap_or(&diag.message);
        report = report.with_label(
            Label::new((filename, diag.span.clone()))
                .with_message(label_text)
                .with_color(color),
        );

        report
            .finish()
            .write((filename, Source::from(source)), &mut output)
            .ok();
    }

    String::from_utf8(output).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_display() {
        let d = Diagnostic::error(0..5, "entity not found: \"Kael\"");
        assert_eq!(d.to_string(), "error: entity not found: \"Kael\"");
    }

    #[test]
    fn render_produces_output() {
        let source = "Kael is a character {\n    member of Unknown\n}";
        let diags = vec![
            Diagnostic::error(35..42, "undefined entity reference")
                .with_label("not defined anywhere"),
        ];
        let output = render_diagnostics(source, "test.ww", &diags);
        assert!(!output.is_empty());
        assert!(output.contains("undefined entity reference"));
    }
}
