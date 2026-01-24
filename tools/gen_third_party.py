#!/usr/bin/env python3
"""
Generate THIRD_PARTY.md (or NOTICE.md) for DIRECT Rust dependencies only.

Requires:
  - Python 3.9+
  - cargo in PATH

How it works:
  - Uses `cargo metadata --format-version 1` to get a resolved dependency graph.
  - Collects direct deps of workspace default members (or all workspace members).
  - Reads actual LICENSE/NOTICE files from each dependency's source directory.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Set, Tuple


LICENSE_CANDIDATE_PATTERNS = [
    "LICENSE", "LICENSE.*", "LICENCE", "LICENCE.*",
    "COPYING", "COPYING.*",
    "NOTICE", "NOTICE.*",
    "COPYRIGHT", "COPYRIGHT.*",
    "UNLICENSE", "UNLICENSE.*",
]


@dataclass(frozen=True)
class Pkg:
    pkg_id: str
    name: str
    version: str
    manifest_path: Path
    license_expr: Optional[str]
    license_file: Optional[Path]
    repository: Optional[str]
    source: Optional[str]


def run_cargo_metadata(repo: Path) -> dict:
    cmd = ["cargo", "metadata", "--format-version", "1"]
    proc = subprocess.run(
        cmd,
        cwd=str(repo),
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        raise SystemExit(
            f"ERROR: cargo metadata failed (exit {proc.returncode}).\n\nSTDERR:\n{proc.stderr}"
        )
    return json.loads(proc.stdout)


def is_within(path: Path, root: Path) -> bool:
    try:
        path.resolve().relative_to(root.resolve())
        return True
    except Exception:
        return False


def github_anchor(text: str) -> str:
    """
    Approximate GitHub-style markdown heading anchors:
    - lowercase
    - remove punctuation except spaces and hyphens
    - spaces -> hyphens
    - collapse multiple hyphens
    """
    t = text.strip().lower()
    t = re.sub(r"[^\w\s-]", "", t)
    t = re.sub(r"\s+", "-", t)
    t = re.sub(r"-{2,}", "-", t)
    return t


def collect_packages(meta: dict) -> Dict[str, Pkg]:
    out: Dict[str, Pkg] = {}
    for p in meta.get("packages", []):
        pkg_id = p["id"]
        manifest_path = Path(p["manifest_path"])
        license_expr = p.get("license")
        license_file = p.get("license_file")
        license_file_path = Path(license_file) if license_file else None
        out[pkg_id] = Pkg(
            pkg_id=pkg_id,
            name=p["name"],
            version=p["version"],
            manifest_path=manifest_path,
            license_expr=license_expr,
            license_file=license_file_path,
            repository=p.get("repository"),
            source=p.get("source"),
        )
    return out


def collect_resolve_nodes(meta: dict) -> Dict[str, dict]:
    resolve = meta.get("resolve")
    if not resolve:
        raise SystemExit("ERROR: `cargo metadata` did not include a `resolve` graph.")
    nodes = resolve.get("nodes", [])
    return {n["id"]: n for n in nodes}


def wanted_dep_kind(dep_kind: Optional[str], include_dev: bool, include_build: bool) -> bool:
    # cargo metadata uses: "normal", "dev", "build" (or null in some versions)
    if dep_kind is None or dep_kind == "normal":
        return True
    if dep_kind == "dev":
        return include_dev
    if dep_kind == "build":
        return include_build
    return False


def direct_deps_of_roots(
    meta: dict,
    pkgs: Dict[str, Pkg],
    nodes: Dict[str, dict],
    repo_root: Path,
    include_dev: bool,
    include_build: bool,
    include_optional: bool,
) -> List[Pkg]:
    workspace_members: Set[str] = set(meta.get("workspace_members", []))
    default_members: List[str] = meta.get("workspace_default_members") or list(workspace_members)

    direct_ids: Set[str] = set()

    for root_id in default_members:
        node = nodes.get(root_id)
        if not node:
            continue
        for dep in node.get("deps", []):
            dep_id = dep["pkg"]
            dep_kinds = dep.get("dep_kinds", [])

            # Filter by kind (normal/dev/build) and optionality (if available).
            ok_kind = any(
                wanted_dep_kind(k.get("kind"), include_dev, include_build) for k in dep_kinds
            )
            if not ok_kind:
                continue

            # Optional deps are tricky: cargo metadata doesn't always expose "optional" here.
            # Best-effort: if it's optional in the root's Cargo.toml but not enabled, it won't appear.
            if not include_optional:
                # If it appears in resolve deps, it is enabled; so we keep it.
                pass

            direct_ids.add(dep_id)

    # Exclude first-party crates in the repo (workspace/path members typically live inside repo root)
    result: List[Pkg] = []
    for dep_id in direct_ids:
        pkg = pkgs.get(dep_id)
        if not pkg:
            continue
        # If the dependency source is None AND its manifest is within the repo, treat as first-party.
        if pkg.source is None and is_within(pkg.manifest_path, repo_root):
            continue
        result.append(pkg)

    # Sort alphabetically
    result.sort(key=lambda p: (p.name.lower(), p.version))
    return result


def find_license_files(pkg_dir: Path) -> List[Path]:
    found: List[Path] = []
    for pat in LICENSE_CANDIDATE_PATTERNS:
        for p in pkg_dir.glob(pat):
            if p.is_file():
                found.append(p)
    # De-dup by resolved path
    uniq: Dict[Path, Path] = {}
    for p in found:
        try:
            uniq[p.resolve()] = p
        except Exception:
            uniq[p] = p
    # stable order
    return sorted(uniq.values(), key=lambda x: x.name.lower())


def safe_read_text(path: Path, max_bytes: int = 512_000) -> str:
    data = path.read_bytes()
    if len(data) > max_bytes:
        data = data[:max_bytes] + b"\n\n[TRUNCATED]\n"
    # Try utf-8, fall back to latin-1
    try:
        return data.decode("utf-8", errors="replace")
    except Exception:
        return data.decode("latin-1", errors="replace")


def render_markdown(
    deps: List[Pkg],
    repo: Path,
    out_name: str,
    include_dev: bool,
    include_build: bool,
    include_optional: bool,
) -> str:
    now = dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")
    kinds = ["normal"]
    if include_dev:
        kinds.append("dev")
    if include_build:
        kinds.append("build")
    kinds_str = ", ".join(kinds)

    header = [
        f"# Third-Party Notices",
        "",
        f"_Generated: {now} (UTC)_",
        "",
        f"Generated by `tools/gen_third_party.py` from direct dependencies only.",
        f"- Repo: `{repo}`",
        f"- Included dependency kinds: **{kinds_str}**",
        f"- Included optional deps: **{include_optional}**",
        "",
        "## Table of contents",
        "",
    ]

    toc_lines = []
    for p in deps:
        title = f"{p.name} {p.version}"
        anchor = github_anchor(title)
        toc_lines.append(f"- [{title}](#{anchor})")

    body = []
    for p in deps:
        title = f"{p.name} {p.version}"
        body.append(f"## {title}")
        body.append("")
        if p.license_expr:
            body.append(f"- License expression (Cargo.toml): `{p.license_expr}`")
        else:
            body.append("- License expression (Cargo.toml): _not specified_")
        if p.repository:
            body.append(f"- Repository: {p.repository}")
        if p.source:
            body.append(f"- Source: `{p.source}`")
        body.append("")

        pkg_dir = p.manifest_path.parent

        license_files: List[Path] = []
        if p.license_file:
            # license_file can be relative to the crate manifest directory
            lf = p.license_file
            if not lf.is_absolute():
                lf = (pkg_dir / lf).resolve()
            if lf.exists() and lf.is_file():
                license_files.append(lf)

        # If no explicit license-file, scan common names
        if not license_files:
            license_files = find_license_files(pkg_dir)

        if not license_files:
            body.append(
                "_No LICENSE/NOTICE file found in the crate source directory. "
                "You may need to fetch the license text from the upstream repository or rely on SPDX templates._"
            )
            body.append("")
            continue

        for lf in license_files:
            rel = None
            try:
                rel = lf.relative_to(pkg_dir)
            except Exception:
                rel = lf
            body.append(f"### {rel}")
            body.append("")
            body.append("```text")
            body.append(safe_read_text(lf).rstrip())
            body.append("```")
            body.append("")

    return "\n".join(header + toc_lines + [""] + body).rstrip() + "\n"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("repo", nargs="?", default=".", help="Path to the Rust repository root")
    ap.add_argument(
        "--output",
        default="THIRD_PARTY.md",
        help="Output markdown filename (e.g. THIRD_PARTY.md or NOTICE.md)",
    )
    ap.add_argument("--include-dev", action="store_true", help="Include dev-dependencies")
    ap.add_argument("--include-build", action="store_true", help="Include build-dependencies")
    ap.add_argument("--include-optional", action="store_true", help="Include optional deps (best-effort)")
    args = ap.parse_args()

    repo = Path(args.repo).resolve()
    meta = run_cargo_metadata(repo)
    pkgs = collect_packages(meta)
    nodes = collect_resolve_nodes(meta)

    deps = direct_deps_of_roots(
        meta=meta,
        pkgs=pkgs,
        nodes=nodes,
        repo_root=repo,
        include_dev=args.include_dev,
        include_build=args.include_build,
        include_optional=args.include_optional,
    )

    md = render_markdown(
        deps=deps,
        repo=repo,
        out_name=args.output,
        include_dev=args.include_dev,
        include_build=args.include_build,
        include_optional=args.include_optional,
    )

    out_path = repo.parent / args.output
    out_path.write_text(md, encoding="utf-8")
    print(f"Wrote {out_path} ({len(deps)} direct third-party deps).")


if __name__ == "__main__":
    main()
