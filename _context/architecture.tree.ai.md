./ {.github, adapters, core, docs, examples, hooks}
.github/ {workflows}
adapters/ {archidoc, archidoc-md, archidoc-rust, archidoc-ts}
adapters/archidoc/ {bin, scripts, tests}
adapters/archidoc-md/ {src}
adapters/archidoc-rust/ {src, tests}
adapters/archidoc-ts/ {src, tests}
adapters/archidoc-ts/tests/ {fixtures}
adapters/archidoc-ts/tests/fixtures/ {dashboard}
adapters/archidoc-ts/tests/fixtures/dashboard/ {charts}
core/ {archidoc-cli, archidoc-engine, archidoc-types, spec, tests}
core/archidoc-cli/ {src, tests}
core/archidoc-engine/ {src}
core/archidoc-engine/src/ {folder_scaffold, init_cmd}
core/archidoc-types/ {src}
core/tests/ {src, tests}
core/tests/src/ {drivers, dsl, fakes}
core/tests/src/drivers/ {in_memory, protocol_driver}
examples/ {rust-example, ts-example}
examples/rust-example/ {src}
examples/rust-example/src/ {api, database, events}
examples/ts-example/ {src}
examples/ts-example/src/ {dashboard, websocket}
