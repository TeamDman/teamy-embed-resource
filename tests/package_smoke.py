"""Check both crate archives and extracted consumers without publishing.

Usage: python3 tests/package_smoke.py [--allow-dirty]
The local patch stands in for the unpublished teamy-facet-toml registry entry.
"""

import argparse
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import tomllib


ROOT = Path(__file__).resolve().parents[1]
PARSER = ROOT / "vendor" / "teamy-facet-toml"


def run(args: list[str], *, cwd: Path, env: dict[str, str]) -> str:
    result = subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"{' '.join(args)} failed:\n{result.stdout}{result.stderr}")
    return result.stdout


def extract(archive: Path, directory: Path, required: set[str]) -> Path:
    with tarfile.open(archive, "r:gz") as tar:
        names = {member.name for member in tar.getmembers()}
        prefix = archive.name.removesuffix(".crate")
        missing = {f"{prefix}/{name}" for name in required} - names
        if missing:
            raise AssertionError(f"{archive.name} omits {sorted(missing)}")
        tar.extractall(directory, filter="data")
    return directory / prefix


def consumer(directory: Path, root_package: Path, parser_package: Path, *, lean: bool) -> None:
    directory.mkdir()
    (directory / "src").mkdir()
    (directory / "src" / "main.rs").write_text("fn main() {}\n")
    api = "compile_for_everything" if lean else "compile"
    (directory / "build.rs").write_text(
        "fn main() { let _api = |path: &str| embed_resource::"
        + api
        + "(path, embed_resource::NONE); }\n"
    )
    (directory / "Cargo.toml").write_text(
        '[package]\nname = "package-smoke-consumer"\nversion = "0.0.0"\nedition = "2024"\n\n'
        "[build-dependencies]\n"
        f'embed-resource = {{ package = "teamy-embed-resource", path = "{root_package}", default-features = {str(not lean).lower()} }}\n\n'
        "[patch.crates-io]\n"
        f'teamy-facet-toml = {{ path = "{parser_package}" }}\n'
    )


def main() -> None:
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("--allow-dirty", action="store_true")
    settings = args.parse_args()

    with tempfile.TemporaryDirectory(prefix="teamy-embed-packaging-") as temporary:
        temp = Path(temporary)
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(temp / "target")
        dirty = ["--allow-dirty"] if settings.allow_dirty else []

        run(["cargo", "package", "--offline", *dirty, "--manifest-path", str(PARSER / "Cargo.toml")], cwd=ROOT, env=env)
        patch = f'patch.crates-io.teamy-facet-toml.path="{PARSER}"'
        run(["cargo", "--config", patch, "package", "--offline", *dirty], cwd=ROOT, env=env)

        archives = temp / "target" / "package"
        parser_package = extract(
            archives / "teamy-facet-toml-0.50.0-rc.7.crate",
            temp,
            {"Cargo.toml", "README.md", "UPSTREAM.md", "LICENSE-MIT", "LICENSE-APACHE", "src/parser.rs", "src/semantic.rs"},
        )
        root_package = extract(
            archives / "teamy-embed-resource-0.1.0.crate",
            temp,
            {"Cargo.toml", "README.md", "PACKAGING.md", "LICENSE", "LICENSE-MIT-UPSTREAM", "src/lib.rs"},
        )
        parser_manifest = tomllib.loads((parser_package / "Cargo.toml").read_text())
        root_manifest = tomllib.loads((root_package / "Cargo.toml").read_text())
        assert parser_manifest["package"]["publish"] == ["crates-io"]
        assert root_manifest["package"]["publish"] is False
        assert root_manifest["package"]["name"] == "teamy-embed-resource"
        assert root_manifest["package"]["version"] == "0.1.0"
        assert root_manifest["lib"]["name"] == "embed_resource"
        parser_dependency = root_manifest["dependencies"]["facet-toml"]
        assert parser_dependency["package"] == "teamy-facet-toml"
        assert parser_dependency["version"] == "=0.50.0-rc.7"
        assert "path" not in parser_dependency

        run(["cargo", "test", "--offline", "--manifest-path", str(parser_package / "Cargo.toml")], cwd=temp, env=env)
        for lean in (True, False):
            mode = "lean" if lean else "default"
            project = temp / f"consumer-{mode}"
            consumer(project, root_package, parser_package, lean=lean)
            run(["cargo", "check", "--offline", "--manifest-path", str(project / "Cargo.toml")], cwd=temp, env=env)
            tree = run(["cargo", "tree", "--offline", "-e", "normal,build", "--prefix", "none", "--manifest-path", str(project / "Cargo.toml")], cwd=temp, env=env)
            if lean:
                assert not any(name in tree for name in ("teamy-facet", "toml_parser", "toml_edit", "serde"))
            else:
                assert "teamy-facet-toml v0.50.0-rc.7" in tree
                assert "toml_parser v1.0.10" in tree
                assert "toml_edit v0.23.7" in tree

    print("parser and root archives verified; extracted lean/default consumers passed with a local parser patch")
    print("nothing was published; the root archive still requires unpublished teamy-facet-toml")


if __name__ == "__main__":
    main()
