# 2026-09-28 — toolchain v0.0.6 (4deba32)

The first row of `bench/history.jsonl`: `infs` and `infc`
[v0.0.6](https://github.com/Inferara/inference/releases/tag/v0.0.6), workload
`pair-math-1` (sha256 `abbe4bcc…`), soroban-sdk 28.0.0 with soroban-env-host
28.0.2. The contract, `out/main.wasm` (sha256 `c94ccd43…`; the row has every
hash), is kept as `bench/modules/main-4deba32.wasm`.

| module | wasm_size | code_size | baseline instructions | baseline mem_bytes |
|---|---:|---:|---:|---:|
| out/main.wasm | 9,409 | 5,613 | 96,531 | 143,441 |

| method | typical Δ instructions | typical Wasm fuel | worst Δ instructions | worst Wasm fuel |
|---|---:|---:|---:|---:|
| minimum_liquidity (baseline) | 0 | 77 | 0 | 77 |
| get_amount_out | 72,194 | 18,118 | 94,034 | 23,578 |
| get_amount_in | 71,246 | 17,881 | 93,118 | 23,349 |
| quote | 58,908 | 14,910 | 85,092 | 21,456 |
| mint_initial | 62,734 | 15,757 | 63,594 | 15,972 |
| mint_proportional | 124,724 | 31,248 | 173,412 | 43,420 |
| burn_share | 61,138 | 15,356 | 85,482 | 21,442 |
| k_holds | 11,144 | 2,967 | 11,220 | 2,986 |
| k_holds_with_fee | 29,276 | 7,386 | 29,364 | 7,408 |
| mul_div | 62,308 | 15,760 | 85,052 | 21,446 |
| mul_div_up | 64,138 | 16,106 | 87,030 | 21,829 |

Every call but the baseline uses 143,441 memory bytes plus 32 per argument
(143,505 to 143,601), and 107,675 to 269,943 instructions in all.

Method. `tests/src/bin/cost.rs` runs each case of the workload
(`tests/src/workload.rs`: per method a typical case, V2's reference values
where one exists, and the input with the most Wasm fuel a search found) under
the in-process Soroban host, in an `Env` that holds only this contract, right
after a `minimum_liquidity()` call, the baseline; `ledger::MEASUREMENT` states
this, and the workload's hash covers it, so a change to how cases are
measured is a new workload. Δ instructions is the case's
`resources().instructions` less the baseline call's. Most of the baseline is
the VM instantiation every call pays, which grows with the module, and an
overhead a toolchain adds to every export's entry is in the baseline too:
both cancel in the Δ. A larger module shows in the first table and in the
absolute instructions and memory bytes, not in a Δ. Under this host a Δ is 4
instructions per unit of Wasm fuel above the baseline's 77, plus a fixed term
per method for its name and its arguments (−424 to +40 here, the same for
both cases). Wasm fuel (`wasm_insns` in the row) is not an instruction count:
it is the wasmi fuel the case consumed, the host's `WasmInsnExec` count, at
the weights soroban-env-host calibrates: 1 per instruction, 2 per load, 1 per
store, 3 per global access, 67 per call and 1 per 8 locals of each function
called, with a block, loop or `if` charged when it is entered. Calls are 30%
to 49% of every case's fuel here but the baseline's (its one call is 67 of its
77), so a toolchain that inlines a helper lowers the fuel by about 67 per call
it removes while executing nearly the same instructions. The memory bytes are
host memory (the VM instance and the arguments), not anything the contract
executes. The host meters deterministically: `bench/snapshot.sh` runs the
binary twice and requires identical output, and a debug build, a release build
and Rust 1.91 and 1.97.1 print the same bytes.

Two rows compare only when they name the same workload (by its sha256), the
same soroban-sdk and the same soroban-env-host: the fixed term in every Δ
comes from how the SDK's test client passes the call to the host. Later
sections link the compiler changes between two rows' toolchains as
`Inferara/inference@<a>...<b>`. The command below compares the last two rows.
It names both toolchain commits and both kept modules, then prints what
changed: the module's sizes that changed (`sizes: unchanged` when none did),
whether the interface changed, and each cost of a case that changed (`no cost
changed` when none did), showing a cost of a method or case that one row lacks
as `null`. It refuses a ledger of fewer than two rows and two rows that do not
compare:

```sh
jq -s -r '
  if length < 2 then "the ledger has fewer than two rows\n" | halt_error(1) else .[-2:] end
  | . as [$a, $b]
  | if [$a.workload_sha256, $a.soroban_sdk, $a.soroban_env_host]
      != [$b.workload_sha256, $b.soroban_sdk, $b.soroban_env_host]
    then "not comparable: workload \($a.workload) -> \($b.workload), soroban-sdk \($a.soroban_sdk)"
      + " -> \($b.soroban_sdk), soroban-env-host \($a.soroban_env_host) -> \($b.soroban_env_host)\n"
      | halt_error(1)
    else . end
  | def changes($fields; $x; $y):
      [$fields[] as $f | select($x[$f] != $y[$f]) | "\($f) \($x[$f]) -> \($y[$f])"];
    "\($a.toolchain_commit)...\($b.toolchain_commit), module \($a.module) -> \($b.module)",
    (changes(["wasm_size", "code_size"]; $a; $b) | "sizes: " + if . == [] then "unchanged" else join(", ") end),
    "interface_sha256: \(if $a.interface_sha256 == $b.interface_sha256 then "unchanged" else "CHANGED" end)",
    ( [ ([$a.cost, $b.cost] | map(keys) | add | unique)[] as $m
        | ([$a.cost[$m], $b.cost[$m]] | map(. // {} | keys) | add | unique)[] as $c
        | changes(["wasm_insns", "delta_instructions", "instructions", "mem_bytes"]; $a.cost[$m][$c] // {};
            $b.cost[$m][$c] // {})
        | select(. != []) | "\($m) \($c): " + join(", ") ]
      | if . == [] then "no cost changed" else .[] end )' \
  bench/history.jsonl
```
