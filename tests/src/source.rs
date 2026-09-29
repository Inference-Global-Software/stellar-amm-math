//! The repository's files, and what the tests read out of the contract's
//! Inference sources: the spec blocks and the refusal table.

use std::path::{Path, PathBuf};

/// This crate's directory when the binary was compiled.
const BUILT_IN: &str = env!("CARGO_MANIFEST_DIR");

/// The repository root: this crate lives in its `tests/` directory.
///
/// # Panics
///
/// When cargo runs this binary for another checkout (see
/// [`assert_built_here`]).
#[must_use]
pub fn repo_root() -> PathBuf {
    assert_built_here();
    Path::new(BUILT_IN).join("..")
}

/// Panics when cargo runs this binary for another checkout than the one it
/// was compiled in.
///
/// Cargo leaves a package's path out of its artifacts' fingerprints, so
/// checkouts that share a target directory can reuse each other's test and
/// `cost` binaries as fresh. Such a binary embeds the other checkout's
/// `out/main.wasm` and reads the other checkout's files, and so tests and
/// measures code this checkout may not have. Cargo sets `CARGO_MANIFEST_DIR`
/// for the binaries it runs (`cargo test`, `cargo run`), which gives the reuse
/// away; a binary run directly, without cargo, is not checked.
///
/// # Panics
///
/// On that mismatch, with the two directories and the remedy.
pub fn assert_built_here() {
    let Some(running) = std::env::var_os("CARGO_MANIFEST_DIR") else {
        return;
    };
    let running = Path::new(&running);
    let profile = if cfg!(debug_assertions) { "" } else { " --release" };
    assert!(
        same_directory(running, Path::new(BUILT_IN)),
        "this binary was built in {BUILT_IN}, but cargo runs it for {}: it reused another checkout's \
         build from a shared target directory. Give each checkout its own CARGO_TARGET_DIR, or run \
         `cargo clean -p stellar-amm-math-tests{profile}` here",
        running.display()
    );
}

fn same_directory(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// The bytes of a file, by its path from the repository root.
///
/// # Panics
///
/// When the file cannot be read; the message names the file and how to build
/// it.
#[must_use]
pub fn read_repo_file(path: &str) -> Vec<u8> {
    let full = repo_root().join(path);
    std::fs::read(&full).unwrap_or_else(|err| {
        panic!(
            "cannot read {path} ({err}); `infs build` writes out/main.wasm and \
             tests/shell/out/main.wasm, and proofs/generate.sh writes proofs/"
        )
    })
}

/// A file from the repository root as text, its `\r\n` line ends read as
/// `\n` (a Windows checkout may convert them).
///
/// # Panics
///
/// When the file cannot be read or is not UTF-8.
#[must_use]
pub fn read_repo_text(path: &str) -> String {
    let text = String::from_utf8(read_repo_file(path)).unwrap_or_else(|err| panic!("{path} is not UTF-8: {err}"));
    text.replace("\r\n", "\n")
}

/// A spec block of the contract's sources: its file, its name, and the
/// prefix of the Rocq definitions the proof build gives its spec functions.
#[derive(Clone, Copy, Debug)]
pub struct SpecBlock {
    pub file: &'static str,
    pub name: &'static str,
    pub rocq_prefix: &'static str,
}

/// Every spec block of `src/`, in the order `proofs/main.v` states their
/// theorems. The tests mirror exactly these; `repo.rs` checks that `src/`
/// declares no other.
pub const SPEC_BLOCKS: [SpecBlock; 2] = [
    SpecBlock { file: "src/main.inf", name: "PairMath", rocq_prefix: "main__PairMath" },
    SpecBlock { file: "src/model.inf", name: "WideArith", rocq_prefix: "main__model_WideArith" },
];

/// The names of the spec blocks `source` declares: every line that starts
/// with `spec `, whatever follows the name.
#[must_use]
pub fn spec_block_names(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("spec "))
        .map(|rest| rest.trim_start().split(|c: char| !is_identifier_char(c)).next().unwrap_or_default())
        .collect()
}

/// A spec function: its name, the Rocq definition its comment names
/// (`Rocq: <definition>.`), if any, and its code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecFunction {
    pub name: String,
    pub rocq: Option<String>,
    /// The function from its header line to its closing brace, without
    /// comments, blank lines or indentation, and with each run of spaces as
    /// one: what it states, whatever the layout.
    pub body: String,
}

/// The spec functions of `spec <block> { ... }` in `source`, in declaration
/// order. The block runs from its opening line to the first line that is a
/// lone `}` at column 0, and every line in it that starts with `fn ` declares
/// a spec function, which runs to the first `}` line at its header's
/// indentation.
///
/// # Panics
///
/// When the block or a spec function in it is missing or unterminated, or
/// when a spec function is not of the one form the mirrors in
/// `tests/specs.rs` evaluate: a parameterless `forall` whose header line is
/// `fn <name>() forall {`.
#[must_use]
pub fn spec_functions(source: &str, block: &str) -> Vec<SpecFunction> {
    let opening = format!("spec {block} {{");
    let mut lines = source.lines().skip_while(|line| line.trim() != opening);
    assert!(lines.next().is_some(), "no `{opening}` line");
    let mut functions = Vec::new();
    let mut rocq = None;
    while let Some(line) = lines.next() {
        if line == "}" {
            return functions;
        }
        let trimmed = line.trim();
        if let Some(tag) = rocq_tag(trimmed) {
            rocq = Some(tag);
        }
        if let Some(header) = trimmed.strip_prefix("fn ") {
            let name = forall_name(header).unwrap_or_else(|| {
                panic!(
                    "`{trimmed}` in `spec {block}`: tests/specs.rs mirrors only parameterless `forall` \
                     spec functions, declared as `fn <name>() forall {{`; give this form a mirror first"
                )
            });
            let closing = format!("{}}}", &line[..line.len() - line.trim_start().len()]);
            let mut body = vec![code_of(line)];
            loop {
                let line = lines.next().filter(|&line| line == closing || line != "}");
                let line = line.unwrap_or_else(|| panic!("`fn {name}` in `spec {block}` has no `{closing}` line"));
                body.push(code_of(line));
                if line == closing {
                    break;
                }
            }
            body.retain(|code| !code.is_empty());
            functions.push(SpecFunction { name: name.to_owned(), rocq: rocq.take(), body: body.join("\n") });
        }
    }
    panic!("`{opening}` has no closing `}}` line");
}

/// A line of Inference without its `//` comment, its indentation and its
/// trailing spaces, and with each run of spaces in it as one.
fn code_of(line: &str) -> String {
    let code = line.split_once("//").map_or(line, |(code, _)| code);
    code.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The pin a reviewed text is recorded under: the first 16 hex digits of its
/// sha256, enough to notice that it changed.
#[must_use]
pub fn review_pin(text: &str) -> String {
    crate::sha256_hex(text.as_bytes())[..16].to_owned()
}

/// The refusal table of `src/main.inf` with the notes under it: the lines
/// from the one that starts `// Refusals.` to the first bare `//` after it,
/// each with its line end.
#[must_use]
pub fn refusal_table(source: &str) -> Option<String> {
    let mut lines = source.lines().skip_while(|line| !line.starts_with("// Refusals."));
    let first = lines.next()?;
    let mut table = String::new();
    for line in std::iter::once(first).chain(lines.take_while(|&line| line != "//")) {
        table.push_str(line);
        table.push('\n');
    }
    Some(table)
}

/// The name in a spec function header `<name>() forall {` (the text after
/// `fn `), spaces allowed between the tokens.
fn forall_name(header: &str) -> Option<&str> {
    let (name, rest) = header.split_once('(')?;
    let name = name.trim();
    let rest = rest.trim_start().strip_prefix(')')?.trim_start().strip_prefix("forall")?;
    let is_identifier = !name.is_empty() && name.chars().all(is_identifier_char);
    (is_identifier && rest.trim() == "{").then_some(name)
}

fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// The definition a comment line names as `Rocq: <definition>.`.
fn rocq_tag(line: &str) -> Option<String> {
    let comment = line.strip_prefix("//")?;
    let (_, rest) = comment.split_once("Rocq: ")?;
    let end = rest.find(|c: char| !is_identifier_char(c))?;
    (rest[end..].starts_with('.') && end > 0).then(|| rest[..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_names_and_rocq_tags_in_order() {
        let source = "\
// Rocq: not_in_the_block.
spec S {
    // Scope: fixed values. Rocq: m__S_hspec1.
    fn first() forall {
        assert(true);
    }

    // No tag here.
    fn second()   forall   {
    }
    // Scope: the envelope a < b.
    // Rocq: m__S_hspec3.
    fn third ( ) forall {
    }
}
fn after() forall {
}
";
        let functions = spec_functions(source, "S");
        let found: Vec<(&str, Option<&str>)> =
            functions.iter().map(|f| (f.name.as_str(), f.rocq.as_deref())).collect();
        assert_eq!(
            found,
            [("first", Some("m__S_hspec1")), ("second", None), ("third", Some("m__S_hspec3"))]
        );
        let bodies: Vec<&str> = functions.iter().map(|f| f.body.as_str()).collect();
        assert_eq!(
            bodies,
            ["fn first() forall {\nassert(true);\n}", "fn second() forall {\n}", "fn third ( ) forall {\n}"]
        );
    }

    /// The body of the one spec function of a block, and its pin.
    fn pinned(function: &str) -> (String, String) {
        let body = spec_functions(&format!("spec S {{\n{function}}}\n"), "S").remove(0).body;
        let pin = review_pin(&body);
        (body, pin)
    }

    /// A body keeps what its function states, so that a pin of it changes
    /// with any edit to a claim or an assumption, and with no edit to a
    /// comment or to the layout.
    #[test]
    fn a_body_and_its_pin_change_with_the_code_only() {
        let original = "    fn f() forall {
        let a: u32 = @;
        assume {
            assert(a > 0);
        }
        assert(a < 10);
    }
";
        let (body, pin) = pinned(original);
        assert_eq!(body, "fn f() forall {\nlet a: u32 = @;\nassume {\nassert(a > 0);\n}\nassert(a < 10);\n}");
        assert_eq!(pin.len(), 16);
        let relaid = "    // What f claims.
    fn f()  forall {
        let a: u32 = @; // drawn

        assume {
            assert(a  >  0);
        }
            assert(a < 10);
    }
";
        assert_eq!(pinned(relaid).1, pin, "a comment, a blank line or spacing moved the pin");
        for edited in [
            original.replace("a < 10", "a < 9"),
            original.replace("a < 10", "a <= 10"),
            original.replace("assert(a > 0);\n", ""),
            original.replace("a > 0", "a > 0 && a < 5"),
            original.replace("let a", "let b"),
        ] {
            assert_ne!(pinned(&edited).1, pin, "an edit kept the pin:\n{edited}");
        }
    }

    /// The message [`spec_functions`] panics with on `source`, read as the
    /// spec block `S`.
    fn refusal(source: &str) -> String {
        let payload = std::panic::catch_unwind(|| spec_functions(source, "S"))
            .expect_err(&format!("{source:?} was not refused"));
        let message = payload.downcast_ref::<String>().map(String::as_str);
        message.or_else(|| payload.downcast_ref::<&str>().copied()).unwrap_or_default().to_owned()
    }

    /// A spec function runs to the `}` at its own indentation, and never
    /// past the block's `}`: one the block closes first is unterminated,
    /// even where a `}` at its indentation follows the block.
    #[test]
    fn refuses_an_unterminated_spec_function() {
        let message = refusal("spec S {\n    fn f() forall {\n        assert(true);\n}\nfn g() {\n    }\n}\n");
        assert!(message.contains("`fn f` in `spec S` has no `    }` line"), "refused for another reason: {message}");
    }

    #[test]
    fn reads_the_refusal_table_to_its_first_bare_comment_line() {
        let source = "// Header.\n//\n// Refusals. Each row\n//   m   a > 0   X\n// A note.\n//\n// After.\n";
        let table = "// Refusals. Each row\n//   m   a > 0   X\n// A note.\n";
        assert_eq!(refusal_table(source).as_deref(), Some(table));
        assert_eq!(refusal_table("// No table.\n"), None);
    }

    /// Every other header is refused, not skipped: a skipped spec function
    /// would ship with no mirror.
    #[test]
    fn refuses_every_other_spec_function_form() {
        for header in [
            "fn some_path() exists {",
            "fn one_path() unique {",
            "fn with_input(x: u32) forall {",
            "fn one_line() forall { assert(true); }",
            "fn returns() -> () forall {",
        ] {
            let message = refusal(&format!("spec S {{\n    {header}\n    }}\n}}\n"));
            let expected = format!("`{header}` in `spec S`: tests/specs.rs mirrors only parameterless `forall`");
            assert!(message.contains(&expected), "`{header}` was refused for another reason: {message}");
        }
    }

    #[test]
    fn finds_every_spec_block() {
        let source = "\
// spec Commented {
spec A {
}
  spec B{
}
spec C { fn c() forall { assert(true); } }
fn spec_like() {
}
";
        assert_eq!(spec_block_names(source), ["A", "B", "C"]);
    }
}
