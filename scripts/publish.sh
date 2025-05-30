#!/usr/bin/env bash

cargo publish -p yazi-macro && sleep 30
cargo publish -p yazi-codegen && sleep 30
cargo publish -p yazi-shared && sleep 30
cargo publish -p yazi-ffi && sleep 30
cargo publish -p yazi-fs && sleep 30
cargo publish -p yazi-term && sleep 30
cargo publish -p yazi-config && sleep 30
cargo publish -p yazi-proxy && sleep 30
cargo publish -p yazi-adapter && sleep 30
cargo publish -p yazi-boot && sleep 30
cargo publish -p yazi-binding && sleep 30
cargo publish -p yazi-dds && sleep 30
cargo publish -p yazi-scheduler && sleep 30
cargo publish -p yazi-plugin && sleep 30
cargo publish -p yazi-widgets && sleep 30
cargo publish -p yazi-core && sleep 30
cargo publish -p yazi-fm && sleep 30
cargo publish -p yazi-cli
