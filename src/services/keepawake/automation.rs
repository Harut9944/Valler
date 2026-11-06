//! Automatic keep awake, after the original's rules: stay awake while chosen
//! apps run and/or while the computer is plugged in. With "Any" one matching
//! condition is enough; with "All" every enabled condition must hold. An
//! automatic session ends when its conditions stop holding, but a session the
//! person started by hand is never ended by the automation.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Rules {
    pub enabled: bool,
    /// Stay awake while any of these programs runs (names as shown in Quit apps).
    pub apps: Vec<String>,
    pub apps_enabled: bool,
    /// Stay awake while plugged in.
    pub power_enabled: bool,
    /// Every enabled condition must hold (instead of any one).
    pub require_all: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Condition {
    Power,
    RunningApps,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    None,
    Activate,
    Deactivate,
}

/// What the system looks like right now.
pub struct World<'a> {
    pub running: &'a [String],
    /// None when the computer has no battery (always on mains power).
    pub on_battery: Option<bool>,
}

fn normalize(name: &str) -> String {
    let n = name.trim().to_lowercase();
    n.strip_suffix(".exe").or_else(|| n.strip_suffix(".app")).unwrap_or(&n).to_string()
}

/// The first chosen app that is running, if any.
pub fn running_app<'a>(rules: &'a Rules, running: &[String]) -> Option<&'a str> {
    let running: Vec<String> = running.iter().map(|r| normalize(r)).collect();
    rules.apps.iter().map(String::as_str).find(|a| {
        let a = normalize(a);
        !a.is_empty() && running.contains(&a)
    })
}

pub fn enabled(rules: &Rules) -> Vec<Condition> {
    let mut v = Vec::new();
    if rules.power_enabled {
        v.push(Condition::Power);
    }
    // An empty app list can't be satisfied, so it doesn't count as enabled.
    if rules.apps_enabled && rules.apps.iter().any(|a| !a.trim().is_empty()) {
        v.push(Condition::RunningApps);
    }
    v
}

pub fn matching(rules: &Rules, w: &World) -> Vec<Condition> {
    let mut v = Vec::new();
    if rules.power_enabled && w.on_battery != Some(true) {
        v.push(Condition::Power);
    }
    if rules.apps_enabled && running_app(rules, w.running).is_some() {
        v.push(Condition::RunningApps);
    }
    v
}

pub fn satisfied(rules: &Rules, w: &World) -> bool {
    let m = matching(rules, w);
    if m.is_empty() {
        return false;
    }
    !rules.require_all || m.len() == enabled(rules).len()
}

/// A short reason for the panel, like "Video Call is running".
pub fn reason(rules: &Rules, w: &World) -> Option<String> {
    if !rules.enabled || !satisfied(rules, w) {
        return None;
    }
    match running_app(rules, w.running).filter(|_| rules.apps_enabled) {
        Some(app) => Some(format!("{app} is running")),
        None => Some("Plugged in".into()),
    }
}

pub fn action(rules: &Rules, w: &World, session_active: bool, automatic_session: bool) -> Action {
    if !rules.enabled || !satisfied(rules, w) {
        return if automatic_session { Action::Deactivate } else { Action::None };
    }
    if session_active { Action::None } else { Action::Activate }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(apps: &[&str], power: bool, all: bool) -> Rules {
        Rules { enabled: true, apps: apps.iter().map(|s| s.to_string()).collect(), apps_enabled: !apps.is_empty(), power_enabled: power, require_all: all }
    }

    fn world<'a>(running: &'a [String], on_battery: Option<bool>) -> World<'a> {
        World { running, on_battery }
    }

    #[test]
    fn matches_apps_by_name_ignoring_case_and_extension() {
        let r = rules(&["Zoom", "obs"], false, false);
        let running = vec!["zoom.exe".to_string(), "bash".into()];
        assert_eq!(running_app(&r, &running), Some("Zoom"));
        assert_eq!(running_app(&r, &["OBS.app".to_string()]), Some("obs"));
        assert_eq!(running_app(&r, &["zoomer".to_string()]), None, "whole names only");
    }

    #[test]
    fn any_mode_needs_one_condition() {
        let r = rules(&["zoom"], true, false);
        let none: Vec<String> = vec![];
        assert!(satisfied(&r, &world(&none, Some(false))), "plugged in");
        assert!(satisfied(&r, &world(&["zoom".into()], Some(true))), "app running on battery");
        assert!(!satisfied(&r, &world(&none, Some(true))));
        assert!(satisfied(&r, &world(&none, None)), "desktops without a battery count as plugged in");
    }

    #[test]
    fn all_mode_needs_every_enabled_condition() {
        let r = rules(&["zoom"], true, true);
        assert!(!satisfied(&r, &world(&["zoom".into()], Some(true))));
        assert!(satisfied(&r, &world(&["zoom".into()], Some(false))));
        // An empty app list doesn't block All forever.
        let mut r2 = rules(&[], true, true);
        r2.apps_enabled = true;
        assert!(satisfied(&r2, &world(&[], Some(false))));
    }

    #[test]
    fn starts_and_ends_only_its_own_sessions() {
        let r = rules(&["zoom"], false, false);
        let on = vec!["zoom".to_string()];
        let off: Vec<String> = vec![];
        assert_eq!(action(&r, &world(&on, None), false, false), Action::Activate);
        assert_eq!(action(&r, &world(&on, None), true, true), Action::None, "already running");
        assert_eq!(action(&r, &world(&off, None), true, true), Action::Deactivate);
        assert_eq!(action(&r, &world(&off, None), true, false), Action::None, "a manual session is left alone");
        let mut disabled = r.clone();
        disabled.enabled = false;
        assert_eq!(action(&disabled, &world(&on, None), true, true), Action::Deactivate);
    }

    #[test]
    fn reasons_for_the_panel() {
        let r = rules(&["Zoom"], true, false);
        assert_eq!(reason(&r, &world(&["zoom".into()], Some(true))).as_deref(), Some("Zoom is running"));
        assert_eq!(reason(&r, &world(&[], Some(false))).as_deref(), Some("Plugged in"));
        assert_eq!(reason(&r, &world(&[], Some(true))), None);
    }
}
