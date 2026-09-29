//! Drift between files that must agree: the test shell's copies of wide.inf
//! and model.inf; the spec blocks of `src/`, the Rocq definitions their
//! comments name and those `proofs/main.v` states; the refusal table and the
//! refusals the tests name; this crate's files, its manifest and its
//! `lib.rs`; and the cost ledger, its workloads, the modules it keeps and the
//! build it records.
//!
//! The Rocq checks compare names, order and count only: a `proofs/main.v`
//! whose obligations are stale but whose definitions keep their names passes
//! them. The ledger check catches it: `bench/snapshot.sh` records a row only
//! for sources it has rebuilt all three modules from, and the committed
//! build needs a row that names its sources and its modules. What an
//! obligation states is checked apart from all of these: `specs.rs` pins
//! each spec function's body next to its mirror, so an edited claim fails
//! there until its mirror is reviewed again.

use serde_json::{Map, Value, json};
use stellar_amm_math_tests::host::contract;
use stellar_amm_math_tests::ledger::{self, lock_version};
use stellar_amm_math_tests::method::REFUSAL_TABLE_PIN;
use stellar_amm_math_tests::source::{
    SPEC_BLOCKS, read_repo_file, read_repo_text, refusal_table, repo_root, review_pin, spec_block_names,
    spec_functions,
};
use stellar_amm_math_tests::workload::{self, WORKLOADS};
use stellar_amm_math_tests::{sections, sha256_hex};

#[test]
fn the_shell_copies_are_the_contracts_files() {
    for file in ["wide.inf", "model.inf"] {
        let contract = read_repo_file(&format!("src/{file}"));
        let shell = read_repo_file(&format!("tests/shell/src/{file}"));
        assert!(
            contract == shell,
            "tests/shell/src/{file} differs from src/{file}: copy it over, or the shell tests stale code"
        );
    }
}

/// The names of the files in `dir`, from the repository root, that end in
/// `extension`, sorted.
fn files_in(dir: &str, extension: &str) -> Vec<String> {
    let entries = std::fs::read_dir(repo_root().join(dir)).unwrap_or_else(|err| panic!("cannot list {dir}: {err}"));
    let mut names: Vec<String> = entries
        .map(|entry| entry.expect("a directory entry").file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(extension))
        .collect();
    names.sort();
    names
}

/// `src/` declares exactly the spec blocks [`SPEC_BLOCKS`] lists, each in its
/// file: a spec block the tests do not know of would ship its obligations
/// with no mirror.
#[test]
fn src_declares_only_the_listed_spec_blocks() {
    let mut declared = Vec::new();
    for name in files_in("src", ".inf") {
        let file = format!("src/{name}");
        for block in spec_block_names(&read_repo_text(&file)) {
            declared.push(format!("{file}: spec {block}"));
        }
    }
    let listed: Vec<String> = SPEC_BLOCKS.iter().map(|block| format!("{}: spec {}", block.file, block.name)).collect();
    assert_eq!(declared, listed, "the spec blocks of src/, and source::SPEC_BLOCKS");
}

/// The Nth spec function's comment names `<prefix>_hspecN`, and
/// `proofs/main.v` defines exactly those hassert definitions, in that order,
/// lists each block's in that order, and states the one theorem per block.
#[test]
fn spec_comments_name_the_emitted_rocq_definitions() {
    let proof = read_repo_text("proofs/main.v");
    let mut expected_hasserts = Vec::new();
    for block in SPEC_BLOCKS {
        let functions = spec_functions(&read_repo_text(block.file), block.name);
        assert!(!functions.is_empty(), "{}: `spec {}` declares no spec function", block.file, block.name);
        let prefix = block.rocq_prefix;
        let expected: Vec<String> = (1..=functions.len()).map(|n| format!("{prefix}_hspec{n}")).collect();
        for (function, want) in functions.iter().zip(&expected) {
            assert_eq!(
                function.rocq.as_deref(),
                Some(want.as_str()),
                "{}: the comment above `fn {}` must name the Rocq definition {want}",
                block.file,
                function.name
            );
        }

        let hspec = format!("{prefix}_hspec");
        let list = format!("Definition {prefix}_specs ");
        let list = proof
            .lines()
            .find(|line| line.starts_with(&list))
            .unwrap_or_else(|| panic!("proofs/main.v has no {list}"));
        let listed: Vec<&str> = list.split(['(', ')', ' ']).filter(|token| token.starts_with(&hspec)).collect();
        assert_eq!(listed, expected, "the list {prefix}_specs in proofs/main.v");
        expected_hasserts.extend(expected);
    }

    assert_eq!(hassert_definitions(&proof), expected_hasserts, "the hassert definitions of proofs/main.v");

    let theorems: Vec<&str> = proof
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("Theorem ")?.split_whitespace().next())
        .collect();
    let mut expected_theorems = vec!["valid_main".to_owned()];
    expected_theorems.extend(SPEC_BLOCKS.iter().map(|block| format!("valid_{}", block.rocq_prefix)));
    assert_eq!(theorems, expected_theorems, "the theorems of proofs/main.v, one per spec block");
    for block in SPEC_BLOCKS {
        let prefix = block.rocq_prefix;
        let theorem = format!("Theorem valid_{prefix} : ValidSpec main {prefix}_specs.");
        assert!(proof.lines().any(|line| line == theorem), "proofs/main.v does not state `{theorem}`");
    }
}

/// The names of a Rocq file's hassert definitions, in order: every
/// `Definition` whose type is `hassert` or whose name has `_hspec` in it,
/// however its line is laid out.
fn hassert_definitions(proof: &str) -> Vec<&str> {
    proof
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start().strip_prefix("Definition")?;
            let rest = rest.strip_prefix(char::is_whitespace)?.trim_start();
            let (name, rest) = rest.split_at(rest.find(|c: char| c.is_whitespace() || c == ':').unwrap_or(rest.len()));
            let typed = rest.trim_start().strip_prefix(':').is_some_and(|ty| ty.trim_start().starts_with("hassert"));
            (typed || name.contains("_hspec")).then_some(name)
        })
        .collect()
}

#[test]
fn finds_hassert_definitions_in_any_layout() {
    let proof = "Definition a : hassert :=\n  HA_true.\nDefinition b_hspec0 : hassert := HA_true.\n  Definition \
                 c:hassert:=HA_true.\nDefinition d_hspec9 := HA_true.\nDefinition e : module_func := {|\n\
                 Definition Vi32 i := i.\nDefinitionx : hassert := HA_true.\n";
    assert_eq!(hassert_definitions(proof), ["a", "b_hspec0", "c", "d_hspec9"]);
}

/// The refusal table of `src/main.inf` is the one [`REFUSAL_TABLE_PIN`]
/// pins: a row added, removed or reworded fails here until
/// `method::Method::refusals`, `oracle::Refusal` and the trap matrix are
/// checked against the new table and the pin follows.
#[test]
fn the_refusal_table_is_the_one_the_refusals_follow() {
    let table = refusal_table(&read_repo_text("src/main.inf")).expect("src/main.inf has a `// Refusals.` table");
    let pin = review_pin(&table);
    assert_eq!(
        pin, REFUSAL_TABLE_PIN,
        "the refusal table of src/main.inf changed: check method::Method::refusals, oracle::Refusal and \
         rows::trap_matrix against it, then set method::REFUSAL_TABLE_PIN to {pin}"
    );
}

#[test]
fn the_committed_modules_are_wasm() {
    for path in ["out/main.wasm", "proofs/main.wasm", "tests/shell/out/main.wasm"] {
        let bytes = read_repo_file(path);
        assert!(bytes.starts_with(b"\0asm\x01\0\0\0"), "{path} is not a WebAssembly module ({} bytes)", bytes.len());
    }
    assert!(!read_repo_text("proofs/main.v").trim().is_empty(), "proofs/main.v is empty");
}

/// The tests embed `out/main.wasm` when they compile, and Cargo recompiles
/// them when it changes. This fails when the file changed during the run, or
/// (through [`read_repo_file`]'s check) when cargo runs a binary it reused
/// from another checkout.
#[test]
fn the_tested_contract_is_out_main_wasm() {
    assert!(contract::WASM == read_repo_file("out/main.wasm").as_slice(), "the embedded contract is stale");
}

/// Files this crate shares between test targets as a `mod`, not targets of
/// their own.
const SHARED_MODULES: [&str; 2] = ["reused.rs", "vm.rs"];

/// The `path` of each `[[<kind>]]` table of a Cargo manifest.
fn target_paths<'a>(manifest: &'a str, kind: &str) -> Vec<&'a str> {
    let header = format!("[[{kind}]]");
    let mut table = "";
    manifest
        .lines()
        .filter_map(|line| {
            if line.starts_with('[') {
                table = line.trim_end();
                return None;
            }
            let path = line.strip_prefix("path = \"")?.strip_suffix('"')?;
            (table == header).then_some(path)
        })
        .collect()
}

/// A file of this crate is compiled only where it is declared (Cargo
/// discovers no target here: the manifest sets `autobins = false` and
/// `autotests = false`), so the tests in an undeclared one never run, and
/// nothing says so. Every `*.rs` in the crate's directory must be a
/// `[[test]]` path or a shared module, every `src/*.rs` but `lib.rs` a
/// `pub mod` of `src/lib.rs`, and every `src/bin/*.rs` a `[[bin]]` path.
#[test]
fn every_test_file_is_declared() {
    let manifest = read_repo_text("tests/Cargo.toml");
    let tests = target_paths(&manifest, "test");
    for file in files_in("tests", ".rs") {
        assert!(
            tests.contains(&file.as_str()) || SHARED_MODULES.contains(&file.as_str()),
            "tests/{file} never runs: declare it as a [[test]] in tests/Cargo.toml"
        );
    }
    let lib = read_repo_text("tests/src/lib.rs");
    for file in files_in("tests/src", ".rs").into_iter().filter(|file| file != "lib.rs") {
        let declaration = format!("pub mod {};", file.trim_end_matches(".rs"));
        assert!(
            lib.lines().any(|line| line == declaration),
            "tests/src/{file} is never compiled: declare it as `{declaration}` in tests/src/lib.rs"
        );
    }
    let bins = target_paths(&manifest, "bin");
    for file in files_in("tests/src/bin", ".rs") {
        let path = format!("src/bin/{file}");
        assert!(
            bins.contains(&path.as_str()),
            "tests/{path} is not declared: declare it as a [[bin]] in tests/Cargo.toml"
        );
    }
}

#[test]
fn reads_the_paths_of_one_kind_of_target() {
    let manifest = "[lib]\npath = \"src/lib.rs\"\n\n[[bin]]\nname = \"b\"\npath = \"src/bin/b.rs\"\n\n[[test]]\n\
                    name = \"t\"\npath = \"t.rs\"\n\n[[test]]\npath = \"u.rs\"\n[profile.dev]\npath = \"x.rs\"\n";
    assert_eq!(target_paths(manifest, "test"), ["t.rs", "u.rs"]);
    assert_eq!(target_paths(manifest, "bin"), ["src/bin/b.rs"]);
}

// ---- the cost ledger --------------------------------------------------

#[test]
fn the_workload_is_the_last_listed_one() {
    workload::check().unwrap_or_else(|message| panic!("{message}"));
    for (i, (id, _)) in WORKLOADS.iter().enumerate() {
        assert!(!WORKLOADS[..i].iter().any(|(earlier, _)| earlier == id), "workload::WORKLOADS lists {id} twice");
    }
}

/// A row of the ledger, as `bench/snapshot.sh` writes it.
type Row = Map<String, Value>;

/// The fields of a row, in the order `bench/snapshot.sh` writes them.
const ROW_FIELDS: [&str; 21] = [
    "date",
    "toolchain_version",
    "toolchain_commit",
    "infs_version",
    "module",
    "source_sha256",
    "manifest_sha256",
    "shell_source_sha256",
    "wasm_sha256",
    "wasm_size",
    "code_size",
    "shell_wasm_sha256",
    "proof_v_sha256",
    "proof_wasm_sha256",
    "interface_sha256",
    "soroban_sdk",
    "soroban_env_host",
    "workload",
    "workload_sha256",
    "cost",
    "notes",
];

/// The fields of each case of a row's `cost`.
const COST_FIELDS: [&str; 4] = ["delta_instructions", "instructions", "mem_bytes", "wasm_insns"];

/// The rows of `bench/history.jsonl`, one JSON object per line; none before
/// `bench/snapshot.sh` records the first.
fn ledger_rows() -> Vec<Row> {
    if !repo_root().join("bench/history.jsonl").exists() {
        return Vec::new();
    }
    let ledger = read_repo_text("bench/history.jsonl");
    assert!(ledger.is_empty() || ledger.ends_with('\n'), "bench/history.jsonl does not end in a newline");
    ledger
        .lines()
        .enumerate()
        .map(|(i, line)| match serde_json::from_str(line) {
            Ok(Value::Object(row)) => row,
            Ok(other) => panic!("bench/history.jsonl:{}: {other} is not an object", i + 1),
            Err(err) => panic!("bench/history.jsonl:{}: not JSON: {err}", i + 1),
        })
        .collect()
}

/// Whether `cost` is `method -> case -> {COST_FIELDS}`, all integers.
fn is_cost(cost: &Value) -> bool {
    let is_case = |case: &Value| {
        case.as_object().is_some_and(|case| {
            case.len() == COST_FIELDS.len()
                && COST_FIELDS.iter().all(|field| case.get(*field).is_some_and(Value::is_i64))
        })
    };
    let is_method = |cases: &Value| cases.as_object().is_some_and(|cases| cases.values().all(is_case));
    cost.as_object().is_some_and(|methods| !methods.is_empty() && methods.values().all(is_method))
}

/// Every line of `bench/history.jsonl` is a JSON object with exactly the
/// fields of a row, each of its type: a truncated row, or one with a field
/// missing, added or of another type, fails here, not only where a field is
/// read. (The ledger test below compares the values of the committed
/// build's row.)
#[test]
fn every_ledger_row_has_the_row_fields() {
    let mut expected = ROW_FIELDS.to_vec();
    expected.sort_unstable();
    for (i, row) in ledger_rows().iter().enumerate() {
        let mut fields: Vec<&str> = row.keys().map(String::as_str).collect();
        fields.sort_unstable();
        assert_eq!(fields, expected, "bench/history.jsonl:{}: the fields of a row", i + 1);
        for (field, value) in row {
            let well_typed = match field.as_str() {
                "wasm_size" | "code_size" => value.is_u64(),
                "cost" => is_cost(value),
                _ => value.is_string(),
            };
            assert!(well_typed, "bench/history.jsonl:{}: {field} is {value}", i + 1);
        }
    }
}

/// What a row's measurement is a function of: the module, the workload and
/// the host.
const MEASURED_UNDER: [&str; 4] = ["wasm_sha256", "workload_sha256", "soroban_sdk", "soroban_env_host"];

/// What a row measures, which the fields [`MEASURED_UNDER`] determine.
const MEASURED: [&str; 3] = ["cost", "code_size", "interface_sha256"];

/// Each pair of rows that measured the same module under the same workload
/// and host but record another cost, code size or interface.
fn contradicting_rows(rows: &[Row]) -> Vec<String> {
    let mut found = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        for (j, earlier) in rows[..i].iter().enumerate() {
            if MEASURED_UNDER.iter().all(|&field| row.get(field) == earlier.get(field)) {
                for field in MEASURED.into_iter().filter(|&field| row.get(field) != earlier.get(field)) {
                    found.push(format!("bench/history.jsonl:{} and :{}: {field}", j + 1, i + 1));
                }
            }
        }
    }
    found
}

/// Rows that measured the same module under the same workload, soroban-sdk
/// and soroban-env-host record the same measurement: the host meters
/// deterministically, so another cost there means the harness measured
/// differently from what the workload's id names, and the row belongs under
/// a new workload.
#[test]
fn rows_of_one_module_workload_and_host_agree() {
    let contradicting = contradicting_rows(&ledger_rows());
    assert!(
        contradicting.is_empty(),
        "rows measured the same module under the same workload and host, and differ in what they measured; a \
         change to how the cases are measured needs a new id in workload::WORKLOADS:\n{}",
        contradicting.join("\n")
    );
}

/// `contradicting_rows` compares each measured field under each key field,
/// every one named here rather than read from [`MEASURED`] and
/// [`MEASURED_UNDER`], so that a field dropped from either list fails.
#[test]
fn finds_rows_that_contradict_each_other() {
    let base = json!({"wasm_sha256": "a", "workload_sha256": "w", "soroban_sdk": "1", "soroban_env_host": "2",
                      "cost": {"m": {"c": 1}}, "code_size": 5, "interface_sha256": "i", "date": "d", "notes": ""});
    let base: Row = base.as_object().expect("an object").clone();
    let with = |changes: &[(&str, Value)]| -> Row {
        let mut row = base.clone();
        for (field, value) in changes {
            row.insert((*field).to_owned(), value.clone());
        }
        row
    };
    let remeasured = [("cost", json!({"m": {"c": 2}})), ("code_size", json!(6)), ("interface_sha256", json!("j"))];
    for (field, value) in &remeasured {
        let pair = [base.clone(), with(&[(*field, value.clone())])];
        assert_eq!(contradicting_rows(&pair), [format!("bench/history.jsonl:1 and :2: {field}")], "another {field}");
    }
    for key in ["wasm_sha256", "workload_sha256", "soroban_sdk", "soroban_env_host"] {
        let mut changes = remeasured.to_vec();
        changes.push((key, json!("x")));
        assert_eq!(contradicting_rows(&[base.clone(), with(&changes)]), Vec::<String>::new(), "another {key}");
    }
    let later = with(&[("date", json!("e")), ("notes", json!("again"))]);
    assert_eq!(contradicting_rows(&[base.clone(), later]), Vec::<String>::new(), "another date and notes");
    let later_and_remeasured = with(&[("date", json!("e")), remeasured[0].clone()]);
    let contradicting = [base.clone(), with(&[("wasm_sha256", json!("b"))]), later_and_remeasured];
    assert_eq!(contradicting_rows(&contradicting), ["bench/history.jsonl:1 and :3: cost"], "the first and third rows");
}

/// `bench/snapshot.sh` refuses to append a row that [`contradicting_rows`]
/// would find, from its own copy of the two field lists: its
/// `MEASURED_UNDER` and `MEASURED` lines must name the fields listed here.
#[test]
fn the_snapshot_script_compares_the_same_fields() {
    let script = read_repo_text("bench/snapshot.sh");
    for (name, fields) in [("MEASURED_UNDER", &MEASURED_UNDER[..]), ("MEASURED", &MEASURED[..])] {
        let prefix = format!("{name}='");
        let list = script.lines().find_map(|line| line.strip_prefix(&prefix)?.strip_suffix('\''));
        let list = list.unwrap_or_else(|| panic!("bench/snapshot.sh has no `{name}='[...]'` line"));
        let listed: Vec<String> = serde_json::from_str(list)
            .unwrap_or_else(|err| panic!("bench/snapshot.sh: {name} is not a JSON list of field names: {err}"));
        assert_eq!(listed, fields, "bench/snapshot.sh's {name}, and tests/repo.rs's");
    }
}

/// Each row's contract is kept as `bench/modules/main-<module>.wasm`, the
/// module the row measured, and every kept module is a row's. The kept
/// modules are what the rows are measured again from when the workload
/// changes; a module missing from a commit, or replaced, fails here.
#[test]
fn every_row_keeps_its_module() {
    let mut kept = Vec::new();
    for (i, row) in ledger_rows().iter().enumerate() {
        let line = i + 1;
        let module = row.get("module").and_then(Value::as_str);
        let module = module.unwrap_or_else(|| panic!("bench/history.jsonl:{line}: no module"));
        let file = format!("main-{module}.wasm");
        let path = format!("bench/modules/{file}");
        let bytes = std::fs::read(repo_root().join(&path)).unwrap_or_else(|err| {
            panic!("bench/history.jsonl:{line}: cannot read {path} ({err}); bench/snapshot.sh keeps it with the row")
        });
        let actual = sha256_hex(&bytes);
        let recorded = row.get("wasm_sha256").unwrap_or(&Value::Null);
        assert!(
            recorded.as_str() == Some(actual.as_str()),
            "{path} has sha256 {actual}, but bench/history.jsonl:{line} records wasm_sha256 {recorded}: \
             bench/snapshot.sh keeps the module a row measured, so put that module back"
        );
        kept.push(file);
    }
    if repo_root().join("bench/modules").exists() {
        for file in files_in("bench/modules", ".wasm") {
            assert!(kept.contains(&file), "bench/modules/{file} is the module of no row of bench/history.jsonl");
        }
    }
}

/// Every row of `bench/history.jsonl` names a listed workload with the hash
/// the list gives it: a workload changed under an old id fails here.
#[test]
fn every_ledger_row_names_a_listed_workload() {
    for (i, row) in ledger_rows().iter().enumerate() {
        let id = row.get("workload").and_then(Value::as_str);
        let id = id.unwrap_or_else(|| panic!("bench/history.jsonl:{}: no workload", i + 1));
        let hash = row.get("workload_sha256").and_then(Value::as_str);
        let listed = WORKLOADS.iter().find(|(listed, _)| *listed == id).map(|&(_, hash)| hash);
        assert!(listed.is_some(), "bench/history.jsonl:{}: workload {id} is not in workload::WORKLOADS", i + 1);
        assert_eq!(hash, listed, "bench/history.jsonl:{}: the hash of workload {id}", i + 1);
    }
}

/// The sha256 of the listing `sha256sum` prints for `paths` (from the
/// repository root) in C-locale order, one `<sha256>  <path>` line each, as
/// `bench/snapshot.sh` computes it. Each file is read as text, its `\r\n`
/// line ends as `\n`, so a checkout that converts them hashes the same.
fn listing_sha256(mut paths: Vec<String>) -> String {
    paths.sort();
    let lines: Vec<String> =
        paths.iter().map(|path| format!("{}  {path}\n", sha256_hex(read_repo_text(path).as_bytes()))).collect();
    sha256_hex(lines.concat().as_bytes())
}

/// The `.inf` files of `dir`, as paths from the repository root.
fn inference_sources(dir: &str) -> Vec<String> {
    files_in(dir, ".inf").into_iter().map(|name| format!("{dir}/{name}")).collect()
}

/// The `infc_version` a manifest pins.
fn infc_version_pin(manifest: &str) -> Option<&str> {
    manifest.lines().find_map(|line| line.strip_prefix("infc_version = \"")?.strip_suffix('"'))
}

/// What a row for the committed build records, field by field: every field
/// but the date, the notes, the toolchain's commit, infs's version and the
/// name the module is kept under, which no committed file determines.
fn committed_build() -> Vec<(&'static str, Value)> {
    let lock = read_repo_text("tests/Cargo.lock");
    let host = lock_version(&lock, "soroban-env-host").expect("tests/Cargo.lock pins soroban-env-host once");
    let sdk = lock_version(&lock, "soroban-sdk").expect("tests/Cargo.lock pins soroban-sdk once");
    let manifest = read_repo_text("Inference.toml");
    let toolchain = infc_version_pin(&manifest).expect("Inference.toml pins an infc_version");
    let code = sections::code(contract::WASM).unwrap_or_else(|message| panic!("out/main.wasm: {message}"));
    let interface = sections::interface(contract::WASM).unwrap_or_else(|message| panic!("out/main.wasm: {message}"));
    let cost = ledger::measure_cost().unwrap_or_else(|message| panic!("{message}")).to_string();
    let cost: Value = serde_json::from_str(&cost).expect("the measured cost is JSON");
    let mut shell_sources = inference_sources("tests/shell/src");
    shell_sources.push("tests/shell/Inference.toml".to_owned());
    vec![
        ("wasm_sha256", json!(sha256_hex(contract::WASM))),
        ("wasm_size", json!(contract::WASM.len())),
        ("code_size", json!(code.len())),
        ("interface_sha256", json!(sha256_hex(interface))),
        ("toolchain_version", json!(toolchain)),
        ("soroban_sdk", json!(sdk)),
        ("soroban_env_host", json!(host)),
        ("workload", json!(workload::WORKLOAD_ID)),
        ("workload_sha256", json!(workload::WORKLOAD_SHA256)),
        ("source_sha256", json!(listing_sha256(inference_sources("src")))),
        ("manifest_sha256", json!(sha256_hex(manifest.as_bytes()))),
        ("shell_source_sha256", json!(listing_sha256(shell_sources))),
        ("shell_wasm_sha256", json!(sha256_hex(&read_repo_file("tests/shell/out/main.wasm")))),
        ("proof_v_sha256", json!(sha256_hex(read_repo_text("proofs/main.v").as_bytes()))),
        ("proof_wasm_sha256", json!(sha256_hex(&read_repo_file("proofs/main.wasm")))),
        ("cost", cost),
    ]
}

/// Each case whose costs `recorded` and `measured` differ, as
/// `method case: field old -> new`.
fn cost_changes(recorded: &Value, measured: &Value) -> Vec<String> {
    let mut changes = Vec::new();
    for (method, cases) in measured.as_object().into_iter().flatten() {
        for (case, now) in cases.as_object().into_iter().flatten() {
            let then = &recorded[method][case];
            for field in COST_FIELDS {
                if then[field] != now[field] {
                    changes.push(format!("{method} {case}: {field} {} -> {}", then[field], now[field]));
                }
            }
        }
    }
    changes
}

/// The ledger has a row for the committed build: out/main.wasm under the
/// workload and host versions this crate pins and the toolchain version
/// `Inference.toml` pins, what the `cost` binary measures now, and the
/// sources, manifests, test shell and proof build committed with it.
/// `bench/snapshot.sh` writes such a row only after it has built all three
/// modules from those sources and the host tests have passed on them, so an
/// edited source without rebuilt modules, a rebuilt module without a new row,
/// and a changed host or measurement all fail here. A row does not say that
/// the mirrors in `specs.rs` still state what an edited spec function
/// claims; the body pins there check that.
///
/// A failure here names what changed. A plain `cargo test` skips this test,
/// so that after an edit to the contract it reports the contract's behaviour
/// and not only that the ledger lags behind. `cargo test --locked
/// --no-fail-fast -- --include-ignored` runs it, the command for CI and
/// before a commit.
#[test]
#[ignore = "requires a bench/history.jsonl row for this build (bench/snapshot.sh); run with --include-ignored"]
fn the_ledger_records_the_committed_build() {
    let build = committed_build();
    let rows = ledger_rows();
    let differences = |row: &Row| -> Vec<String> {
        build
            .iter()
            .filter_map(|(field, value)| {
                let recorded = row.get(*field).unwrap_or(&Value::Null);
                (recorded != value).then(|| match *field {
                    "cost" => format!("cost:\n  {}", cost_changes(recorded, value).join("\n  ")),
                    _ => format!("{field}: the row has {recorded}, the build {value}"),
                })
            })
            .collect()
    };
    if rows.iter().any(|row| differences(row).is_empty()) {
        return;
    }
    let wasm = &build[0].1;
    let closest = rows.iter().rev().find(|row| row.get("wasm_sha256") == Some(wasm)).or(rows.last());
    let report = match closest {
        Some(row) => format!(
            "the last row for {}, of {}, differs in\n{}",
            if row.get("wasm_sha256") == Some(wasm) { "this out/main.wasm" } else { "another out/main.wasm" },
            row.get("date").unwrap_or(&Value::Null),
            differences(row).join("\n")
        ),
        None => "the ledger is empty".to_owned(),
    };
    panic!(
        "bench/history.jsonl has no row for the committed build (out/main.wasm {wasm}): rebuild the modules \
         and run bench/snapshot.sh; {report}"
    );
}
