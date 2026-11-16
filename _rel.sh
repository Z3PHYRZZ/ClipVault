#!/usr/bin/env bash
export PATH="/c/Users/Dodot/.cargo/bin:$PATH"
cd "C:/Users/Dodot/Documents/GitHub/ClipVault" || exit 1
cargo build --release > _rel.txt 2>&1
echo "RELEASE=$?" >> _rel.txt
cargo run --example dump_icon --quiet >> _rel.txt 2>&1
echo "ICON=$?" >> _rel.txt
"/c/Users/Dodot/AppData/Local/Programs/Inno Setup 6/ISCC.exe" "installer/ClipVault.iss" >> _rel.txt 2>&1
echo "ISCC=$?" >> _rel.txt
