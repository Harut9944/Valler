//! Package manager and app updates without a terminal. Uses whatever the
//! system has: Flatpak and Homebrew on Linux, Homebrew on macOS, winget on
//! Windows. System package managers (apt, dnf, pacman) need administrator
//! rights and are left to the system's own tools.

use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Backend {
    Flatpak,
    Brew,
    BrewCask,
    Winget,
}

impl Backend {
    pub fn label(self) -> &'static str {
        match self {
            Backend::Flatpak => "Flatpak",
            Backend::Brew => "Homebrew",
            Backend::BrewCask => "Homebrew app",
            Backend::Winget => "winget",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Package {
    pub backend: Backend,
    pub id: String,
    pub name: String,
    pub version: String,
    pub new_version: String,
    pub desc: String,
}

#[derive(Clone, Default)]
pub struct State {
    pub backends: Vec<Backend>,
    pub installed: Option<Vec<Package>>,
    pub updates: Option<Vec<Package>>,
    pub results: Option<Vec<Package>>,
    pub busy: Option<String>,
    pub message: Option<String>,
}

pub type Shared = Arc<Mutex<State>>;

fn has(cmd: &str) -> bool {
    let probe = if cfg!(windows) { "where" } else { "which" };
    Command::new(probe).arg(cmd).output().map(|o| o.status.success()).unwrap_or(false)
}

fn out(cmd: &str, args: &[&str]) -> Option<String> {
    let o = Command::new(cmd).args(args).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).to_string())
}

fn run(cmd: &str, args: &[&str]) -> Result<(), String> {
    let o = Command::new(cmd).args(args).output().map_err(|e| e.to_string())?;
    if o.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&o.stderr).trim().lines().last().unwrap_or("").to_string();
        Err(if err.is_empty() { format!("{cmd} exited with an error") } else { err })
    }
}

pub fn detect() -> Vec<Backend> {
    let mut v = Vec::new();
    if cfg!(target_os = "linux") && has("flatpak") {
        v.push(Backend::Flatpak);
    }
    if !cfg!(windows) && has("brew") {
        v.push(Backend::Brew);
        if cfg!(target_os = "macos") {
            v.push(Backend::BrewCask);
        }
    }
    if cfg!(windows) && has("winget") {
        v.push(Backend::Winget);
    }
    v
}

// ------------------------------------------------------------- flatpak

fn flatpak_rows(text: &str, backend: Backend, updates: bool) -> Vec<Package> {
    text.lines()
        .filter_map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            if c.len() < 2 || c[0].trim().is_empty() {
                return None;
            }
            let version = c.get(2).copied().unwrap_or("").trim().to_string();
            Some(Package {
                backend,
                id: c[0].trim().to_string(),
                name: c[1].trim().to_string(),
                version: if updates { String::new() } else { version.clone() },
                new_version: if updates { version } else { String::new() },
                desc: c.get(3).copied().unwrap_or("").trim().to_string(),
            })
        })
        .collect()
}

// ------------------------------------------------------------- brew

fn brew_list(cask: bool) -> Vec<Package> {
    let args: &[&str] = if cask { &["list", "--cask", "--versions"] } else { &["list", "--formula", "--versions"] };
    parse_brew_list(&out("brew", args).unwrap_or_default(), cask)
}

/// `brew list --versions`: "name 1.2 1.3" per line.
fn parse_brew_list(text: &str, cask: bool) -> Vec<Package> {
    let b = if cask { Backend::BrewCask } else { Backend::Brew };
    text.lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let name = it.next()?.to_string();
            Some(Package { backend: b, id: name.clone(), name, version: it.collect::<Vec<_>>().join(" "), new_version: String::new(), desc: String::new() })
        })
        .collect()
}

fn brew_outdated() -> Vec<Package> {
    let mut v = parse_brew_outdated(&out("brew", &["outdated", "--formula", "--verbose"]).unwrap_or_default(), false);
    v.extend(parse_brew_outdated(&out("brew", &["outdated", "--cask", "--verbose"]).unwrap_or_default(), true));
    v
}

/// `brew outdated --verbose`: "name (1.0) < 1.1" (casks may use "!=").
fn parse_brew_outdated(text: &str, cask: bool) -> Vec<Package> {
    text.lines()
        .filter_map(|l| {
            let (left, new) = l.split_once(" < ").or_else(|| l.split_once(" != "))?;
            let (name, cur) = left.split_once(' ').unwrap_or((left, ""));
            Some(Package {
                backend: if cask { Backend::BrewCask } else { Backend::Brew },
                id: name.to_string(),
                name: name.to_string(),
                version: cur.trim_matches(|c| c == '(' || c == ')').to_string(),
                new_version: new.trim().to_string(),
                desc: String::new(),
            })
        })
        .collect()
}

fn brew_search(q: &str) -> Vec<Package> {
    let mut v = Vec::new();
    for (cask, flag) in [(false, "--formula"), (true, "--cask")] {
        if cask && !cfg!(target_os = "macos") {
            continue;
        }
        for l in out("brew", &["search", flag, q]).unwrap_or_default().lines() {
            let n = l.trim();
            if n.is_empty() || n.starts_with("==>") || n.contains(' ') {
                continue;
            }
            v.push(Package { backend: if cask { Backend::BrewCask } else { Backend::Brew }, id: n.into(), name: n.into(), version: String::new(), new_version: String::new(), desc: String::new() });
        }
    }
    v.truncate(60);
    v
}

// ------------------------------------------------------------- winget

/// winget prints a fixed-width table; columns are found from the header line.
fn winget_table(text: &str, has_available: bool) -> Vec<Package> {
    let lines: Vec<&str> = text.lines().map(|l| l.rsplit('\r').next().unwrap_or(l)).collect();
    let Some(h) = lines.iter().position(|l| l.contains(" Id ") && l.contains("Version")) else { return vec![] };
    let header = lines[h];
    let col = |name: &str| header.find(name);
    let (Some(id_at), Some(ver_at)) = (col(" Id ").map(|i| i + 1), col("Version")) else { return vec![] };
    let avail_at = if has_available { col("Available") } else { None };
    let src_at = col("Source");
    let slice = |l: &str, a: usize, b: Option<usize>| -> String {
        let chars: Vec<char> = l.chars().collect();
        let end = b.unwrap_or(chars.len()).min(chars.len());
        if a >= end { String::new() } else { chars[a..end].iter().collect::<String>().trim().to_string() }
    };
    lines[h + 1..]
        .iter()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('-') && !l.contains("upgrades available"))
        .filter_map(|l| {
            let name = slice(l, 0, Some(id_at));
            let id = slice(l, id_at, Some(ver_at));
            if id.is_empty() || id.contains(' ') {
                return None;
            }
            let version = slice(l, ver_at, avail_at.or(src_at));
            let new_version = avail_at.map(|a| slice(l, a, src_at)).unwrap_or_default();
            Some(Package { backend: Backend::Winget, id, name, version, new_version, desc: String::new() })
        })
        .collect()
}

const WINGET_AGREE: [&str; 2] = ["--accept-source-agreements", "--disable-interactivity"];

// ------------------------------------------------------------- operations

fn set(state: &Shared, f: impl FnOnce(&mut State)) {
    if let Ok(mut s) = state.lock() {
        f(&mut s);
    }
}

fn spawn(state: Shared, ctx: egui::Context, busy: String, f: impl FnOnce(&Shared) + Send + 'static) {
    set(&state, |s| {
        s.busy = Some(busy);
        s.message = None;
    });
    std::thread::spawn(move || {
        f(&state);
        set(&state, |s| s.busy = None);
        ctx.request_repaint();
    });
}

pub fn refresh(state: Shared, ctx: egui::Context) {
    spawn(state, ctx, "Reading installed apps…".into(), |st| {
        let backends = detect();
        let mut installed = Vec::new();
        let mut updates = Vec::new();
        for b in &backends {
            match b {
                Backend::Flatpak => {
                    installed.extend(flatpak_rows(&out("flatpak", &["list", "--app", "--columns=application,name,version"]).unwrap_or_default(), *b, false));
                    updates.extend(flatpak_rows(&out("flatpak", &["remote-ls", "--updates", "--app", "--columns=application,name,version"]).unwrap_or_default(), *b, true));
                }
                Backend::Brew => {
                    installed.extend(brew_list(false));
                    updates.extend(brew_outdated());
                }
                Backend::BrewCask => installed.extend(brew_list(true)),
                Backend::Winget => {
                    let mut args = vec!["list"];
                    args.extend(WINGET_AGREE);
                    installed.extend(winget_table(&out("winget", &args).unwrap_or_default(), true).into_iter().map(|mut p| {
                        p.new_version.clear();
                        p
                    }));
                    let mut args = vec!["upgrade"];
                    args.extend(WINGET_AGREE);
                    updates.extend(winget_table(&out("winget", &args).unwrap_or_default(), true));
                }
            }
        }
        // Flatpak update listings carry the new version only; fill in the current one.
        for u in updates.iter_mut().filter(|u| u.backend == Backend::Flatpak) {
            if let Some(i) = installed.iter().find(|i| i.id == u.id) {
                u.version = i.version.clone();
            }
        }
        installed.sort_by_key(|p| p.name.to_lowercase());
        set(st, |s| {
            s.backends = backends;
            s.installed = Some(installed);
            s.updates = Some(updates);
        });
    });
}

pub fn search(state: Shared, ctx: egui::Context, query: String) {
    spawn(state, ctx, format!("Searching for {query}…"), move |st| {
        let backends = detect();
        let mut results = Vec::new();
        for b in &backends {
            match b {
                Backend::Flatpak => results.extend(flatpak_rows(&out("flatpak", &["search", "--columns=application,name,version,description", &query]).unwrap_or_default(), *b, false)),
                Backend::Brew => results.extend(brew_search(&query)),
                Backend::BrewCask => {}
                Backend::Winget => {
                    let mut args = vec!["search", query.as_str()];
                    args.extend(WINGET_AGREE);
                    results.extend(winget_table(&out("winget", &args).unwrap_or_default(), false));
                }
            }
        }
        results.truncate(80);
        set(st, |s| s.results = Some(results));
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    Install,
    Remove,
    Update,
}

pub fn act(state: Shared, ctx: egui::Context, p: Package, op: Op) {
    let verb = match op {
        Op::Install => "Installing",
        Op::Remove => "Removing",
        Op::Update => "Updating",
    };
    let ctx2 = ctx.clone();
    spawn(state, ctx, format!("{verb} {}…", p.name), move |st| {
        let id = p.id.as_str();
        let r = match (p.backend, op) {
            (Backend::Flatpak, Op::Install) => run("flatpak", &["install", "-y", "--noninteractive", "flathub", id]),
            (Backend::Flatpak, Op::Remove) => run("flatpak", &["uninstall", "-y", "--noninteractive", id]),
            (Backend::Flatpak, Op::Update) => run("flatpak", &["update", "-y", "--noninteractive", id]),
            (Backend::Brew, Op::Install) => run("brew", &["install", "--formula", id]),
            (Backend::BrewCask, Op::Install) => run("brew", &["install", "--cask", id]),
            (Backend::Brew | Backend::BrewCask, Op::Remove) => run("brew", &["uninstall", id]),
            (Backend::Brew | Backend::BrewCask, Op::Update) => run("brew", &["upgrade", id]),
            (Backend::Winget, op) => {
                let verb = match op {
                    Op::Install => "install",
                    Op::Remove => "uninstall",
                    Op::Update => "upgrade",
                };
                let mut args = vec![verb, "--id", id, "-e", "--silent", "--accept-package-agreements"];
                if op == Op::Remove {
                    args.retain(|a| *a != "--accept-package-agreements");
                }
                args.extend(WINGET_AGREE);
                run("winget", &args)
            }
        };
        let done = match op {
            Op::Install => "installed",
            Op::Remove => "removed",
            Op::Update => "updated",
        };
        let msg = match r {
            Ok(_) => format!("{} {done}", p.name),
            Err(e) => format!("Couldn't finish: {e}"),
        };
        set(st, |s| s.message = Some(msg));
    });
    refresh_later(ctx2);
}

pub fn update_all(state: Shared, ctx: egui::Context) {
    spawn(state, ctx, "Updating everything…".into(), |st| {
        let mut errors = Vec::new();
        for b in detect() {
            let r = match b {
                Backend::Flatpak => run("flatpak", &["update", "-y", "--noninteractive"]),
                Backend::Brew => run("brew", &["upgrade"]),
                Backend::BrewCask => Ok(()),
                Backend::Winget => {
                    let mut args = vec!["upgrade", "--all", "--silent", "--accept-package-agreements"];
                    args.extend(WINGET_AGREE);
                    run("winget", &args)
                }
            };
            if let Err(e) = r {
                errors.push(format!("{}: {e}", b.label()));
            }
        }
        set(st, |s| s.message = Some(if errors.is_empty() { "Everything is up to date".into() } else { errors.join("; ") }));
    });
}

/// After an operation the lists are stale; the view refreshes when it sees this.
fn refresh_later(ctx: egui::Context) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("packages-stale"), true));
}

pub fn demo() -> State {
    let p = |id: &str, name: &str, v: &str, nv: &str| Package { backend: Backend::Flatpak, id: id.into(), name: name.into(), version: v.into(), new_version: nv.into(), desc: String::new() };
    State {
        backends: vec![Backend::Flatpak],
        installed: Some(vec![p("org.example.Editor", "Text Editor", "46.1", ""), p("org.example.Player", "Media Player", "3.0.21", ""), p("org.example.Paint", "Paint", "5.2", "")]),
        updates: Some(vec![p("org.example.Player", "Media Player", "3.0.21", "3.0.22")]),
        results: None,
        busy: None,
        message: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_winget_table() {
        let t = "Name            Id                 Version  Available Source\n------------------------------------------------------------\nGit             Git.Git            2.40.0   2.41.0    winget\n";
        let r = winget_table(t, true);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "Git.Git");
        assert_eq!(r[0].new_version, "2.41.0");
    }
    #[test]
    fn parses_flatpak() {
        let r = flatpak_rows("org.a.B\tB app\t1.0\n", Backend::Flatpak, false);
        assert_eq!(r[0].name, "B app");
    }
}

#[cfg(test)]
mod feature_tests {
    use super::*;

    #[test]
    fn parses_homebrew_lists() {
        let l = parse_brew_list("git 2.45.0\nnode 20.1 22.3\n", false);
        assert_eq!(l.len(), 2);
        assert_eq!((l[1].name.as_str(), l[1].version.as_str()), ("node", "20.1 22.3"));
        assert_eq!(parse_brew_list("firefox 130.0\n", true)[0].backend, Backend::BrewCask);
        let o = parse_brew_outdated("git (2.44.0) < 2.45.0\nslack (4.1) != 4.2\nnot a row\n", false);
        assert_eq!(o.len(), 2);
        assert_eq!((o[0].version.as_str(), o[0].new_version.as_str()), ("2.44.0", "2.45.0"));
        assert_eq!(o[1].new_version, "4.2");
    }

    #[test]
    fn parses_winget_search_without_available_column() {
        let t = "Name   Id            Version Match      Source\n---------------------------------------------\nVLC    VideoLAN.VLC  3.0.21  Tag: video winget\n";
        let r = winget_table(t, false);
        assert_eq!(r.len(), 1);
        assert_eq!((r[0].name.as_str(), r[0].id.as_str()), ("VLC", "VideoLAN.VLC"));
        assert!(r[0].new_version.is_empty());
        assert!(winget_table("No package found matching input criteria.", false).is_empty());
    }

    #[test]
    fn parses_flatpak_updates_and_search() {
        let u = flatpak_rows("org.a.App\tApp\t2.0\n", Backend::Flatpak, true);
        assert_eq!((u[0].version.as_str(), u[0].new_version.as_str()), ("", "2.0"));
        let s = flatpak_rows("org.b.B\tB\t1.0\tA nice app\n\n", Backend::Flatpak, false);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].desc, "A nice app");
    }

    #[test]
    fn demo_and_labels() {
        let d = demo();
        assert_eq!(d.installed.as_ref().unwrap().len(), 3);
        assert_eq!(d.updates.as_ref().unwrap()[0].new_version, "3.0.22");
        for b in [Backend::Flatpak, Backend::Brew, Backend::BrewCask, Backend::Winget] {
            assert!(!b.label().is_empty());
        }
    }

    #[test]
    fn detection_does_not_fail() {
        // Whatever this machine has, detecting must not panic or hang.
        let _ = detect();
    }
}
