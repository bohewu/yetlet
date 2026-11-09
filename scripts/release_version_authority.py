"""Accepted public SDK/CLI distribution versions from owner-approved records."""
from pathlib import Path
import re


PUBLICATION_RECORDS = {
    "SDK": ("docs/ALPHA6-RELEASE-PREPARATION.md", "registry consumer"),
    "CLI": ("docs/CLI-020-RELEASE-PREPARATION.md", "installed consumer"),
}


def published_versions(root):
    versions = {}
    for subject, (relative, consumer) in PUBLICATION_RECORDS.items():
        text = (Path(root) / relative).read_text(encoding="utf-8")
        sections = re.split(
            r"(?m)^## Owner-approved publication and " + re.escape(consumer) + r"[^\n]*\n",
            text)
        if len(sections) != 2:
            raise ValueError(f"{relative}: accepted publication section required")
        section = re.split(r"(?m)^## ", sections[1], maxsplit=1)[0]
        prefix = "yetlet-cli-v" if subject == "CLI" else "v"
        values = re.findall(
            r"https://github\.com/bohewu/yetlet/releases/tag/" + prefix +
            r"([0-9][0-9A-Za-z.+-]*)\)", section)
        if len(values) != 1:
            raise ValueError(f"{relative}: one accepted release tag reference required")
        versions[subject] = values[0]
    return versions
