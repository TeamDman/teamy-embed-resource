# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Compare build-script linker directives using a fake windres executable.

Usage: python3 tests/mock_linking.py [path-to-upstream-checkout]
No Windows toolchain or real resource compilation is required.
"""

from pathlib import Path
import os
import subprocess
import sys
import tempfile


FORK = Path(__file__).resolve().parents[1]
UPSTREAM = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else None

CASES = {
    "explicit-bin": {
        "manifest": '''[package]
name = "mock-fixture"
version.workspace = true
edition = "2024"
autobins = false
custom-key = { nested = [1, 2, 3] }

[workspace.package]
version = "0.1.0"

[[bin]]
name = "escaped"
path = "src/bin/esc\\u0061ped.rs"
required-features = []
''',
        "source": "src/bin/escaped.rs",
        "api": "compile",
        "expected": "cargo:rustc-link-arg-bins=",
    },
    "autodiscovered-main": {
        "manifest": '''[package]
name = "mock-fixture"
version = "0.1.0"
edition = "2024"
''',
        "source": "src/main.rs",
        "api": "compile",
        "expected": "cargo:rustc-link-arg-bins=",
    },
    "autodiscovered-bin-dir": {
        "manifest": '''[package]
name = "mock-fixture"
version = "0.1.0"
edition = "2024"
''',
        "source": "src/bin/tool.rs",
        "api": "compile",
        "expected": "cargo:rustc-link-arg-bins=",
    },
    "no-bin": {
        "manifest": '''[package]
name = "mock-fixture"
version = "0.1.0"
edition = "2024"
''',
        "source": "src/lib.rs",
        "api": "compile",
        "expected": "cargo:rustc-link-search=native=",
    },
    "everything-lean": {
        "manifest": '''[package]
name = "mock-fixture"
version = "0.1.0"
edition = "2024"
''',
        "source": "src/main.rs",
        "api": "compile_for_everything",
        "expected": "cargo:rustc-link-arg=",
    },
}


def run_case(root: Path, name: str, case: dict, upstream: bool) -> tuple[str, ...]:
    with tempfile.TemporaryDirectory(prefix="embed-resource-mock-") as temporary:
        directory = Path(temporary)
        (directory / "Cargo.toml").write_text(
            case["manifest"]
            + '\n[build-dependencies]\nembed-resource = { path = "'
            + str(root)
            + '\", default-features = '
            + ("true" if upstream or case["api"] == "compile" else "false")
            + " }\n"
        )
        (directory / case["source"]).parent.mkdir(parents=True, exist_ok=True)
        (directory / case["source"]).write_text("fn main() {}\n" if case["source"].endswith(".rs") and "lib.rs" not in case["source"] else "pub fn fixture() {}\n")
        (directory / "version.rc").write_text('1 ICON "icon.ico"\n')
        (directory / "build.rs").write_text(
            "fn main() { embed_resource::"
            + case["api"]
            + '("version.rc", embed_resource::NONE).manifest_required().unwrap(); }\n'
        )
        fake_rc = directory / "fake-windres"
        fake_rc.write_text(
            '#!/bin/sh\nif [ "$1" = "-V" ]; then echo "GNU windres mock"; fi\nexit 0\n'
        )
        fake_rc.chmod(0o755)
        environment = os.environ.copy()
        environment["RC"] = str(fake_rc)
        environment["CARGO_TARGET_DIR"] = str(directory / "target")
        result = subprocess.run(
            ["cargo", "check", "--offline", "--quiet"],
            cwd=directory,
            env=environment,
            capture_output=True,
            text=True,
        )
        if result.returncode:
            raise AssertionError(f"{name} ({root}) failed:\n{result.stderr}")
        outputs = list((directory / "target/debug/build").glob("mock-fixture-*/output"))
        assert len(outputs) == 1, (name, outputs)
        directives = tuple(
            "cargo:rustc-link-search=native="
            if line.startswith("cargo:rustc-link-search=native=")
            else "cargo:rustc-link-lib=dylib="
            if line.startswith("cargo:rustc-link-lib=dylib=")
            else line.split("=", 1)[0] + "="
            for line in outputs[0].read_text().splitlines()
            if line.startswith("cargo:rustc-link-")
        )
        assert case["expected"] in directives, (name, directives)
        return directives


for case_name, case_config in CASES.items():
    fork_directives = run_case(FORK, case_name, case_config, False)
    if UPSTREAM is not None:
        upstream_directives = run_case(UPSTREAM, case_name, case_config, True)
        assert fork_directives == upstream_directives, (
            case_name,
            fork_directives,
            upstream_directives,
        )
    print(f"{case_name}: {fork_directives}")
