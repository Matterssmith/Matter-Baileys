#!/usr/bin/env python3
"""
Check source upstream revisions. No automatic dependency upgrades or execution
of upstream code. Used by GitHub Actions and offline contract tests.

Créditos:
Hecho por Lournyx Matterssmith [Lou-Nyx].
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import sys
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "config" / "upstream-sources.json"
VALID_SHA = re.compile(r"^[0-9a-f]{40}$")
VALID_REPO = re.compile(r"^[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$")
VALID_BRANCH = re.compile(r"^[A-Za-z0-9._/-]+$")


def validate_sources(config: dict) -> list:
    if type(config) is not dict or config.get("schema") != 1:
        raise ValueError("invalid_manifest")
    sources = config.get("sources")
    if not isinstance(sources, list) or not sources:
        raise ValueError("empty_sources")
    names = set()
    for source in sources:
        if not isinstance(source, dict) or set(source) != {
            "name", "repository", "branch", "observed_sha", "license"
        }:
            raise ValueError("invalid_source")
        if (source["name"] in names
                or not VALID_REPO.fullmatch(source["repository"])
                or not VALID_BRANCH.fullmatch(source["branch"])
                or not VALID_SHA.fullmatch(source["observed_sha"])):
            raise ValueError("unsafe_source")
        names.add(source["name"])
    return sources


def latest_sha(source: dict) -> str:
    repo = source["repository"]
    branch = source["branch"]
    api = f"https://api.github.com/repos/{repo}/commits/{branch}"
    headers = {"Accept": "application/vnd.github+json", "User-Agent": "Matter-upstream-monitor/1"}
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"
    with urlopen(Request(api, headers=headers), timeout=12) as response:
        payload = json.load(response)
    value = payload.get("sha")
    if not isinstance(value, str) or not VALID_SHA.fullmatch(value):
        raise ValueError("invalid_remote_sha")
    return value


def report(config: dict, current: dict[str, str]) -> dict:
    results = []
    for source in validate_sources(config):
        name = source["name"]
        latest = current.get(name)
        if not isinstance(latest, str) or not VALID_SHA.fullmatch(latest):
            raise ValueError("invalid_remote_sha")
        changed = latest != source["observed_sha"]
        results.append({
            "name": name,
            "repository": source["repository"],
            "baseline": source["observed_sha"],
            "latest": latest,
            "changed": changed,
            "compare_url": f"https://github.com/{source['repository']}/compare/{source['observed_sha']}...{latest}",
            "requires_manual_review": changed,
        })
    return {"schema": 1, "updates_detected": sum(item["changed"] for item in results), "results": results}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--offline", action="store_true", help="Only validate manifest using baseline SHAs")
    args = parser.parse_args()
    try:
        config = json.loads(MANIFEST.read_text(encoding="utf-8"))
        sources = validate_sources(config)
        current = {
            source["name"]: (source["observed_sha"] if args.offline else latest_sha(source))
            for source in sources
        }
        print(json.dumps(report(config, current), indent=2, sort_keys=True))
        return 0
    except (ValueError, KeyError, OSError, HTTPError, URLError, json.JSONDecodeError) as exc:
        # Never log tokens or request headers. Errors are a change-monitoring
        # failure, not authority to continue with a guessed upstream SHA.
        print(f"upstream_check_failed: {type(exc).__name__}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
