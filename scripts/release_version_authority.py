"""Accepted public SDK/CLI distribution versions from owner-approved records."""
from pathlib import Path
import re


PUBLICATION_RECORDS = {
    "SDK": ("docs/ALPHA8-HOSTED-RECORD.md", "registry consumer"),
    "CLI": ("docs/CLI-031-HOSTED-RECORD.md", "installed consumer"),
}


def _publications(root):
    publications = {}
    for subject, (relative, consumer) in PUBLICATION_RECORDS.items():
        text = (Path(root) / relative).read_text(encoding="utf-8")
        headings = list(re.finditer(
            r"(?m)^## (Owner-approved publication and " + re.escape(consumer) + r"[^\n]*)\n", text))
        if len(headings) != 1:
            raise ValueError(f"{relative}: accepted publication section required")
        heading = headings[0]
        section = re.split(r"(?m)^## ", text[heading.end():], maxsplit=1)[0]
        prefix = "yetlet-cli-v" if subject == "CLI" else "v"
        values = re.findall(
            r"https://github\.com/bohewu/yetlet/releases/tag/" + prefix +
            r"([0-9][0-9A-Za-z.+-]*)\)", section)
        if len(values) != 1:
            raise ValueError(f"{relative}: one accepted release tag reference required")
        anchor = re.sub(r'[^\w-]', '', heading[1].lower().replace(' ', '-'))
        publications[subject] = (values[0], relative + '#' + anchor)
    return publications


def published_versions(root):
    return {subject: value[0] for subject, value in _publications(root).items()}


def publication_links(root):
    return {subject: value[1] for subject, value in _publications(root).items()}
