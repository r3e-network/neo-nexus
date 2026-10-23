//! The platform model behind the first CI contract rule.
//!
//! A literal executable path is a promise that the program exists on the
//! machine that runs it. That promise is only keepable when the platform
//! dependency is visible in the source, which means a compile-time gate: a
//! literal may only be reachable on the platforms where the executable
//! actually exists.
//!
//! The model has four sample platforms — one per CI matrix entry plus the
//! second Windows toolchain — and every `cfg` predicate is evaluated against
//! all four at once. A predicate that cannot be decided (`feature = "…"`,
//! `test`, an unknown name) counts as "may hold everywhere", so a gate that
//! does not talk about platforms never narrows the scope and therefore never
//! hides a platform dependency.

use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Expr, ExprCall, Lit, Meta, MetaList, MetaNameValue, Path, Token};

/// A bitmask over the sample platforms.
pub type PlatformSet = u8;

/// Windows with the MSVC toolchain.
pub const WINDOWS_MSVC: PlatformSet = 1 << 0;
/// Windows with the GNU toolchain.
pub const WINDOWS_GNU: PlatformSet = 1 << 1;
/// Linux with the GNU toolchain.
pub const LINUX_GNU: PlatformSet = 1 << 2;
/// macOS on Apple silicon.
pub const MACOS_AARCH64: PlatformSet = 1 << 3;

/// Every sample platform.
pub const ALL_PLATFORMS: PlatformSet = WINDOWS_MSVC | WINDOWS_GNU | LINUX_GNU | MACOS_AARCH64;
/// The Unix samples.
pub const UNIX_PLATFORMS: PlatformSet = LINUX_GNU | MACOS_AARCH64;
/// The Windows samples.
pub const WINDOWS_PLATFORMS: PlatformSet = WINDOWS_MSVC | WINDOWS_GNU;

/// The sample platforms in a stable order, with the name used in reports.
const PLATFORM_NAMES: [(PlatformSet, &str); 4] = [
    (WINDOWS_MSVC, "windows-msvc"),
    (WINDOWS_GNU, "windows-gnu"),
    (LINUX_GNU, "linux-gnu"),
    (MACOS_AARCH64, "macos-aarch64"),
];

/// The names of the platforms in a set, for a report.
pub fn describe(set: PlatformSet) -> String {
    let names: Vec<&str> = PLATFORM_NAMES
        .iter()
        .filter(|(bit, _)| set & bit != 0)
        .map(|(_, name)| *name)
        .collect();
    if names.is_empty() {
        "no platform".to_string()
    } else {
        names.join(", ")
    }
}

/// What a `cfg` predicate can be known to mean.
///
/// Both halves are needed. `may_be_true` is where the predicate could hold,
/// which is what decides whether code behind it can be compiled at all.
/// `definitely_true` is where it must hold, which is what `not` needs: the
/// platforms where a predicate is false are the platforms where its negation
/// is true.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Evaluation {
    /// Platforms where the predicate may hold.
    pub may_be_true: PlatformSet,
    /// Platforms where the predicate definitely holds.
    pub definitely_true: PlatformSet,
}

impl Evaluation {
    /// A predicate nobody can decide: it may hold everywhere and holds
    /// nowhere.
    fn unknown() -> Self {
        Self {
            may_be_true: ALL_PLATFORMS,
            definitely_true: 0,
        }
    }

    /// A predicate with a known answer.
    fn exact(set: PlatformSet) -> Self {
        Self {
            may_be_true: set,
            definitely_true: set,
        }
    }

    /// `all(a, b)`: both must hold.
    fn all(left: Self, right: Self) -> Self {
        Self {
            may_be_true: left.may_be_true & right.may_be_true,
            definitely_true: left.definitely_true & right.definitely_true,
        }
    }

    /// `any(a, b)`: one of them must hold.
    fn any(left: Self, right: Self) -> Self {
        Self {
            may_be_true: left.may_be_true | right.may_be_true,
            definitely_true: left.definitely_true | right.definitely_true,
        }
    }

    /// `not(a)`: where `a` cannot hold.
    fn not(inner: Self) -> Self {
        Self {
            may_be_true: ALL_PLATFORMS & !inner.definitely_true,
            definitely_true: ALL_PLATFORMS & !inner.may_be_true,
        }
    }
}

/// Evaluate a `cfg` predicate against the sample platforms.
pub fn evaluate_meta(meta: &Meta) -> Evaluation {
    match meta {
        Meta::Path(path) => evaluate_path(path),
        Meta::NameValue(value) => evaluate_name_value(value),
        Meta::List(list) => evaluate_list(list),
    }
}

/// `unix` and `windows` are the two bare names that carry a platform.
fn evaluate_path(path: &Path) -> Evaluation {
    let Some(segment) = path.segments.last() else {
        return Evaluation::unknown();
    };
    match segment.ident.to_string().as_str() {
        "unix" => Evaluation::exact(UNIX_PLATFORMS),
        "windows" => Evaluation::exact(WINDOWS_PLATFORMS),
        _ => Evaluation::unknown(),
    }
}

/// The `target_*` keys describe the sample platforms exactly.
fn evaluate_name_value(value: &MetaNameValue) -> Evaluation {
    let Some(segment) = value.path.segments.last() else {
        return Evaluation::unknown();
    };
    let key = segment.ident.to_string();
    let Expr::Lit(literal) = &value.value else {
        return Evaluation::unknown();
    };
    let Lit::Str(text) = &literal.lit else {
        return Evaluation::unknown();
    };
    match (key.as_str(), text.value().as_str()) {
        ("target_os", "windows") => Evaluation::exact(WINDOWS_PLATFORMS),
        ("target_os", "linux") => Evaluation::exact(LINUX_GNU),
        ("target_os", "macos") => Evaluation::exact(MACOS_AARCH64),
        ("target_family", "windows") => Evaluation::exact(WINDOWS_PLATFORMS),
        ("target_family", "unix") => Evaluation::exact(UNIX_PLATFORMS),
        ("target_env", "msvc") => Evaluation::exact(WINDOWS_MSVC),
        ("target_env", "gnu") => Evaluation::exact(WINDOWS_GNU | LINUX_GNU),
        ("target_env", "") => Evaluation::exact(MACOS_AARCH64),
        ("target_arch", "x86_64") => Evaluation::exact(WINDOWS_PLATFORMS | LINUX_GNU),
        ("target_arch", "aarch64") => Evaluation::exact(MACOS_AARCH64),
        ("target_pointer_width", "64") => Evaluation::exact(ALL_PLATFORMS),
        ("target_pointer_width", "32") => Evaluation::exact(0),
        ("target_vendor", "apple") => Evaluation::exact(MACOS_AARCH64),
        ("target_vendor", "unknown") => Evaluation::exact(WINDOWS_PLATFORMS | LINUX_GNU),
        ("target_endian", "little") => Evaluation::exact(ALL_PLATFORMS),
        ("target_endian", "big") => Evaluation::exact(0),
        // A platform key the sample set does not model: the code is not
        // compiled anywhere in CI, so it cannot break a CI job.
        (
            "target_os"
            | "target_family"
            | "target_env"
            | "target_arch"
            | "target_pointer_width"
            | "target_vendor"
            | "target_endian",
            _,
        ) => Evaluation::exact(0),
        _ => Evaluation::unknown(),
    }
}

/// `all(…)`, `any(…)` and `not(…)` are the only list forms that decide a
/// predicate; any other list is a key nobody here understands.
fn evaluate_list(list: &MetaList) -> Evaluation {
    let name = list
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
        .unwrap_or_default();
    let items = parse_meta_list(list);
    match name.as_str() {
        "all" => items
            .iter()
            .map(evaluate_meta)
            .fold(Evaluation::exact(ALL_PLATFORMS), Evaluation::all),
        "any" => items
            .iter()
            .map(evaluate_meta)
            .fold(Evaluation::exact(0), Evaluation::any),
        "not" => match items.first() {
            Some(inner) => Evaluation::not(evaluate_meta(inner)),
            None => Evaluation::unknown(),
        },
        _ => Evaluation::unknown(),
    }
}

/// Parse the body of a `cfg(…)` attribute into its predicates.
pub fn parse_meta_list(list: &MetaList) -> Vec<Meta> {
    Punctuated::<Meta, Token![,]>::parse_terminated
        .parse2(list.tokens.clone())
        .map(|punctuated| punctuated.into_iter().collect())
        .unwrap_or_default()
}

/// Where an executable exists.
pub fn executable_availability(literal: &str) -> Option<PlatformSet> {
    let lowered = literal.to_ascii_lowercase();
    if lowered.starts_with("/bin/")
        || lowered.starts_with("/sbin/")
        || lowered.starts_with("/usr/bin/")
        || lowered.starts_with("/usr/sbin/")
        || lowered.starts_with("/usr/local/bin/")
    {
        return Some(UNIX_PLATFORMS);
    }
    if lowered.starts_with("c:\\windows")
        || lowered.starts_with("c:/windows")
        || lowered.starts_with("c:\\program files")
        || lowered.starts_with("c:/program files")
        || lowered.starts_with("%systemroot%")
    {
        return Some(WINDOWS_PLATFORMS);
    }
    let name = lowered.rsplit(['/', '\\']).next().unwrap_or(&lowered);
    match name {
        // Shells and utilities that only exist on Unix.
        "sh" | "bash" | "dash" | "zsh" | "ksh" | "ash" | "sleep" | "kill" | "pkill" | "killall"
        | "chmod" | "chown" | "chgrp" | "chroot" | "ps" | "df" | "du" | "uname" | "id"
        | "whoami" | "sed" | "awk" | "grep" | "egrep" | "xargs" | "sudo" | "su" | "nohup"
        | "strace" | "gdb" | "valgrind" | "lsof" | "apt" | "apt-get" | "yum" | "dnf" | "apk"
        | "brew" | "pacman" | "systemctl" | "launchctl" | "osascript" | "head" | "tail" | "wc"
        | "uniq" | "cat" => Some(UNIX_PLATFORMS),

        // Shells and tools that only exist on Windows.
        "cmd" | "cmd.exe" | "powershell" | "powershell.exe" | "pwsh" | "pwsh.exe" | "taskkill"
        | "taskkill.exe" | "icacls" | "icacls.exe" | "attrib" | "attrib.exe" | "netsh"
        | "netsh.exe" | "tasklist" | "tasklist.exe" | "ipconfig" | "ipconfig.exe"
        | "systeminfo" | "systeminfo.exe" | "shutdown" | "shutdown.exe" | "where" | "where.exe"
        | "reg" | "reg.exe" | "wmic" | "wmic.exe" | "robocopy" | "diskpart" | "bcdedit"
        | "cipher" | "cipher.exe" | "assoc" | "ftype" | "del" | "erase" | "xcopy" | "tracert"
        | "pathping" | "schtasks" | "net" => Some(WINDOWS_PLATFORMS),

        // `timeout` is a GNU coreutils program on Linux and `timeout.exe` on
        // Windows, but macOS ships neither.
        "timeout" => Some(LINUX_GNU | WINDOWS_PLATFORMS),

        _ => None,
    }
}

/// The platforms where a piece of code may be compiled but the executable it
/// spawns does not exist.
pub fn missing_platforms(literal: &str, scope: PlatformSet) -> PlatformSet {
    match executable_availability(literal) {
        Some(available) => scope & (ALL_PLATFORMS & !available),
        None => 0,
    }
}

/// A platform-specific executable that a piece of code spawns by literal.
pub struct Spawn {
    /// The literal handed to the process builder.
    pub literal: String,
    /// The platforms where the surrounding code may be compiled.
    pub scope: PlatformSet,
    /// Line of the call, one-based.
    pub line: usize,
    /// Column of the call, zero-based.
    pub column: usize,
}

/// Walks a parsed file and records every platform-specific executable that is
/// spawned by literal, together with the platforms the surrounding code may
/// be compiled for.
#[derive(Default)]
pub struct PlatformGateVisitor {
    /// One entry per scope-bearing node that is currently open. The scope of
    /// a call is the intersection of every entry.
    scopes: Vec<PlatformSet>,
    spawns: Vec<Spawn>,
}

impl PlatformGateVisitor {
    /// A visitor with the file scope in place.
    pub fn new() -> Self {
        Self {
            scopes: vec![ALL_PLATFORMS],
            spawns: Vec::new(),
        }
    }

    /// The literal spawns found so far.
    pub fn spawns(&self) -> &[Spawn] {
        &self.spawns
    }

    fn push_scope(&mut self) {
        self.scopes.push(ALL_PLATFORMS);
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// The platforms the code at the top of the stack may be compiled for.
    fn scope(&self) -> PlatformSet {
        self.scopes
            .iter()
            .fold(ALL_PLATFORMS, |scope, frame| scope & frame)
    }

    /// Narrow the innermost scope by a gate.
    fn narrow(&mut self, evaluation: Evaluation) {
        if let Some(frame) = self.scopes.last_mut() {
            *frame &= evaluation.may_be_true;
        }
    }
}

/// Every node kind that carries attributes can carry a `cfg`, and a `cfg` on
/// any of them narrows the code beneath it. Each one therefore opens a scope
/// while it is visited.
macro_rules! scoped_visitors {
    ($($method:ident : $ty:ty),* $(,)?) => {
        $(
            fn $method(&mut self, node: &'ast $ty) {
                self.push_scope();
                syn::visit::$method(self, node);
                self.pop_scope();
            }
        )*
    };
}

impl<'ast> Visit<'ast> for PlatformGateVisitor {
    scoped_visitors! {
        visit_item: syn::Item,
        visit_expr: syn::Expr,
        visit_local: syn::Local,
        visit_local_init: syn::LocalInit,
        visit_stmt_macro: syn::StmtMacro,
        visit_impl_item: syn::ImplItem,
        visit_trait_item: syn::TraitItem,
        visit_foreign_item: syn::ForeignItem,
        visit_item_foreign_mod: syn::ItemForeignMod,
        visit_field: syn::Field,
        visit_variant: syn::Variant,
        visit_arm: syn::Arm,
        visit_field_value: syn::FieldValue,
        visit_pat: syn::Pat,
        visit_field_pat: syn::FieldPat,
        visit_type_param: syn::TypeParam,
        visit_const_param: syn::ConstParam,
        visit_lifetime_param: syn::LifetimeParam,
        visit_derive_input: syn::DeriveInput,
        visit_receiver: syn::Receiver,
        visit_variadic: syn::Variadic,
        visit_abi: syn::Abi,
        visit_bare_fn_arg: syn::BareFnArg,
        visit_bare_variadic: syn::BareVariadic,
        visit_type_tuple: syn::TypeTuple,
    }

    /// A `cfg` attribute is the only attribute that decides where code is
    /// compiled.
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        if attribute.path().is_ident("cfg") {
            if let Meta::List(list) = &attribute.meta {
                for meta in parse_meta_list(list) {
                    self.narrow(evaluate_meta(&meta));
                }
            }
        }
        syn::visit::visit_attribute(self, attribute);
    }

    /// A process builder call is the shape the rule is about.
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        // The call carries its own attributes, and they scope the call.
        for attribute in &node.attrs {
            self.visit_attribute(attribute);
        }
        if let Some(literal) = spawned_literal(&node.func, &node.args) {
            if executable_availability(&literal).is_some() {
                let start = node.func.span().start();
                self.spawns.push(Spawn {
                    literal,
                    scope: self.scope(),
                    line: start.line,
                    column: start.column,
                });
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

/// The string literal a process builder is handed, if it is handed one.
///
/// `Command::new` may be spelled with any number of leading path segments,
/// and the literal may be an ordinary or a raw string; both are the same
/// promise to the operating system.
fn spawned_literal(
    func: &Expr,
    args: &syn::punctuated::Punctuated<Expr, Token![,]>,
) -> Option<String> {
    let Expr::Path(path) = func else {
        return None;
    };
    let segments: Vec<String> = path
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    if segments.len() < 2 {
        return None;
    }
    let tail = &segments[segments.len() - 2..];
    if tail[0] != "Command" || tail[1] != "new" {
        return None;
    }
    let first = args.first()?;
    let Expr::Lit(literal) = first else {
        return None;
    };
    let Lit::Str(value) = &literal.lit else {
        return None;
    };
    Some(value.value())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scope a snippet's spawn is compiled under.
    fn scope_of(source: &str) -> PlatformSet {
        let file = syn::parse_file(source).expect("the snippet parses");
        let mut visitor = PlatformGateVisitor::new();
        visitor.visit_file(&file);
        visitor
            .spawns()
            .first()
            .map(|spawn| spawn.scope)
            .unwrap_or(ALL_PLATFORMS)
    }

    /// A snippet that spawns a Unix-only executable behind the given gate.
    fn spawns_sh(gate: &str) -> PlatformSet {
        scope_of(&format!(
            "{gate}\nfn f() {{ let _ = std::process::Command::new(\"sh\"); }}"
        ))
    }

    #[test]
    fn a_bare_call_is_compiled_everywhere() {
        assert_eq!(spawns_sh(""), ALL_PLATFORMS);
    }

    #[test]
    fn a_unix_gate_excludes_windows() {
        assert_eq!(spawns_sh("#[cfg(unix)]"), UNIX_PLATFORMS);
    }

    #[test]
    fn a_negated_windows_gate_keeps_unix() {
        assert_eq!(spawns_sh("#[cfg(not(windows))]"), UNIX_PLATFORMS);
    }

    #[test]
    fn an_unknown_gate_never_narrows_the_scope() {
        assert_eq!(
            spawns_sh("#[cfg(feature = \"windows-only\")]"),
            ALL_PLATFORMS
        );
        assert_eq!(
            spawns_sh("#[cfg(not(feature = \"anything\"))]"),
            ALL_PLATFORMS
        );
    }

    #[test]
    fn a_nested_gate_is_the_intersection() {
        assert_eq!(
            spawns_sh("#[cfg(unix)]\n#[cfg(target_os = \"linux\")]"),
            LINUX_GNU
        );
    }

    #[test]
    fn a_gate_on_a_statement_scopes_the_statement() {
        let source =
            "fn f() {\n    #[cfg(windows)]\n    let _ = std::process::Command::new(\"sh\");\n}";
        assert_eq!(scope_of(source), WINDOWS_PLATFORMS);
    }

    #[test]
    fn an_ordinary_string_literal_is_recognised() {
        let file =
            syn::parse_file("fn f() { let _ = std::process::Command::new(\"/bin/sleep\"); }")
                .expect("the snippet parses");
        let mut visitor = PlatformGateVisitor::new();
        visitor.visit_file(&file);
        assert_eq!(visitor.spawns().len(), 1);
        assert_eq!(visitor.spawns()[0].literal, "/bin/sleep");
    }

    #[test]
    fn an_ungated_literal_is_missing_on_every_other_platform() {
        assert_eq!(missing_platforms("sh", ALL_PLATFORMS), WINDOWS_PLATFORMS);
        assert_eq!(missing_platforms("sh", UNIX_PLATFORMS), 0);
        assert_eq!(missing_platforms("cmd", WINDOWS_PLATFORMS), 0);
        assert_eq!(missing_platforms("cmd", UNIX_PLATFORMS), UNIX_PLATFORMS);
    }

    #[test]
    fn a_gated_literal_is_accepted() {
        assert_eq!(missing_platforms("sh", UNIX_PLATFORMS), 0);
    }

    #[test]
    fn a_literal_outside_its_platform_is_reported() {
        assert_eq!(
            missing_platforms("sh", WINDOWS_PLATFORMS),
            WINDOWS_PLATFORMS
        );
    }

    #[test]
    fn a_literal_that_is_not_platform_specific_is_never_missing() {
        assert_eq!(missing_platforms("rustc", ALL_PLATFORMS), 0);
        assert_eq!(
            missing_platforms("not-started-by-this-test", ALL_PLATFORMS),
            0
        );
    }
}
