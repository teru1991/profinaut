use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateSeverity {
    Warn,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateIssue {
    pub severity: GateSeverity,
    pub code: &'static str,
    pub message: String,
    pub context: BTreeMap<String, String>,
}

impl GateIssue {
    pub fn warn(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: GateSeverity::Warn,
            code,
            message: message.into(),
            context: BTreeMap::new(),
        }
    }

    pub fn fail(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: GateSeverity::Fail,
            code,
            message: message.into(),
            context: BTreeMap::new(),
        }
    }

    pub fn with_ctx(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.context.insert(k.into(), v.into());
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GateReport {
    pub issues: Vec<GateIssue>,
}

impl GateReport {
    pub fn push(&mut self, issue: GateIssue) {
        self.issues.push(issue);
    }

    pub fn has_failures(&self) -> bool {
        self.issues.iter().any(|i| i.severity == GateSeverity::Fail)
    }

    pub fn failures_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == GateSeverity::Fail)
            .count()
    }

    pub fn warnings_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == GateSeverity::Warn)
            .count()
    }

    pub fn fail_if_any_failures(&self) -> Result<(), String> {
        if !self.has_failures() {
            return Ok(());
        }
        Err(self.format_human_readable())
    }

    pub fn format_human_readable(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "SSOT Integrity Gate v2: failures={}, warnings={}\n",
            self.failures_count(),
            self.warnings_count()
        ));

        for (idx, issue) in self.issues.iter().enumerate() {
            let sev = match issue.severity {
                GateSeverity::Warn => "WARN",
                GateSeverity::Fail => "FAIL",
            };
            out.push_str(&format!(
                "{:03} [{}] {}: {}\n",
                idx + 1,
                sev,
                issue.code,
                issue.message
            ));
            if !issue.context.is_empty() {
                for (k, v) in &issue.context {
                    out.push_str(&format!("      - {k}: {v}\n"));
                }
            }
        }
        out
    }
}
