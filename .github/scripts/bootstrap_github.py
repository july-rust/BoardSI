#!/usr/bin/env python3
"""Safe, idempotent GitHub planning bootstrap; requires workflow 'issues: write'."""
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

base = "https://api.github.com/repos/" + os.environ["GITHUB_REPOSITORY"]
token = os.environ["GITHUB_TOKEN"]
plan = json.loads(Path("docs/project-plan.json").read_text(encoding="utf-8"))
headers = {"Accept": "application/vnd.github+json", "Authorization": "Bearer " + token,
           "X-GitHub-Api-Version": "2022-11-28", "Content-Type": "application/json"}

def api(method, endpoint, payload=None):
    data = json.dumps(payload).encode() if payload is not None else None
    request = urllib.request.Request(base + endpoint, data=data, method=method, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=35) as response:
            raw = response.read()
            return json.loads(raw) if raw else None
    except urllib.error.HTTPError as error:
        raise RuntimeError(f"{method} {endpoint}: HTTP {error.code}: {error.read().decode(errors='replace')[:700]}") from error

def all_items(path):
    out, page = [], 1
    while True:
        batch = api("GET", path + ("&" if "?" in path else "?") + "per_page=100&page=" + str(page))
        if not isinstance(batch, list):
            raise RuntimeError("Expected list at " + path)
        out.extend(batch)
        if len(batch) != 100:
            return out
        page += 1

def main():
    existing_labels = {x["name"] for x in all_items("/labels")}
    for label in plan["labels"]:
        if label["name"] not in existing_labels:
            api("POST", "/labels", label)
            print("Created label:", label["name"])
    milestone_by_title = {x["title"]: x for x in all_items("/milestones?state=all")}
    for item in plan["milestones"]:
        if item["title"] not in milestone_by_title:
            milestone_by_title[item["title"]] = api("POST", "/milestones",
                 {"title": item["title"], "description": item["description"], "state": "open"})
            print("Created milestone:", item["title"])
    number_by_phase = {x["code"]: milestone_by_title[x["title"]]["number"] for x in plan["milestones"]}
    existing_issues = {x["title"]: x for x in all_items("/issues?state=all") if "pull_request" not in x}
    for item in plan["issues"]:
        issue = existing_issues.get(item["title"])
        if issue is None:
            print("Missing roadmap issue, skipped:", item["title"])
            continue
        n = issue["number"]
        m = number_by_phase[item["phase"]]
        if issue.get("milestone") is None or issue["milestone"]["number"] != m:
            api("PATCH", f"/issues/{n}", {"milestone": m})
        required = [item["type"], "area:" + item["area"], item["priority"]]
        current = {x["name"] for x in issue.get("labels", [])}
        missing = [x for x in required if x not in current]
        if missing:
            api("POST", f"/issues/{n}/labels", {"labels": missing})
        print("Linked issue", n, "to", item["phase"])
    print("Milestone/label sync finished. A GitHub Projects v2 board is NOT created here.")

if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, ValueError, KeyError) as e:
        print("Bootstrap ERROR:", e, file=sys.stderr)
        sys.exit(1)
