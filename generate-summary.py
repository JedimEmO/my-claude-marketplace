#!/usr/bin/env python3
"""
Auto-generates mdbook SUMMARY.md by walking plugins/.

Discovers all .md files, extracts H1 titles, and organizes them
by plugin > skill > references. Outputs to stdout.
"""

import os
import re
from pathlib import Path

PLUGINS_DIR = Path("plugins")
SKIP_DIRS = {".claude-plugin", ".git", "node_modules", "__pycache__"}
SKIP_FILES = {"SUMMARY.md"}


def extract_title(filepath: Path) -> str:
    """Extract the first H1 heading from a markdown file, or derive from filename."""
    try:
        with open(filepath) as f:
            for line in f:
                line = line.strip()
                # Skip YAML frontmatter
                if line == "---":
                    in_frontmatter = True
                    for line in f:
                        if line.strip() == "---":
                            break
                    continue
                # Match # Title
                m = re.match(r"^#\s+(.+)", line)
                if m:
                    return m.group(1).strip()
    except (OSError, UnicodeDecodeError):
        pass

    # Fallback: humanize the filename
    name = filepath.stem
    if name == "SKILL":
        return filepath.parent.name.replace("-", " ").title()
    return name.replace("-", " ").replace("_", " ").title()


def find_md_files(directory: Path) -> list[Path]:
    """Find all .md files under directory, excluding skipped dirs/files."""
    results = []
    for root, dirs, files in os.walk(directory):
        dirs[:] = [d for d in sorted(dirs) if d not in SKIP_DIRS]
        for f in sorted(files):
            if f.endswith(".md") and f not in SKIP_FILES:
                results.append(Path(root) / f)
    return results


def main():
    print("# Summary\n")
    print("[Introduction](index.md)\n")

    if not PLUGINS_DIR.exists():
        return

    for plugin_dir in sorted(PLUGINS_DIR.iterdir()):
        if not plugin_dir.is_dir() or plugin_dir.name.startswith("."):
            continue

        plugin_name = plugin_dir.name
        skills_dir = plugin_dir / "skills"

        # Collect plugin-level md files (not in skills/)
        plugin_files = [
            f for f in find_md_files(plugin_dir)
            if "skills" not in f.relative_to(plugin_dir).parts[:1]
               or not (plugin_dir / "skills").exists()
        ]

        print(f"# {plugin_name.replace('-', ' ').title()}\n")

        if not skills_dir.exists():
            # Plugin with no skills subdir — just list all md files
            for md in find_md_files(plugin_dir):
                rel = md.relative_to(plugin_dir)
                title = extract_title(md)
                print(f"- [{title}]({plugin_name}/{rel})")
            print()
            continue

        # Organize by skill
        for skill_dir in sorted(skills_dir.iterdir()):
            if not skill_dir.is_dir() or skill_dir.name.startswith("."):
                continue

            skill_name = skill_dir.name

            # Find SKILL.md (the main file)
            skill_md = skill_dir / "SKILL.md"
            if skill_md.exists():
                title = extract_title(skill_md)
                rel = skill_md.relative_to(plugin_dir)
                print(f"- [{title}]({plugin_name}/{rel})")
            else:
                print(f"- [{skill_name.replace('-', ' ').title()}]()")

            # Find reference files
            refs = [
                f for f in find_md_files(skill_dir)
                if f != skill_md
            ]
            for ref_md in refs:
                ref_title = extract_title(ref_md)
                rel = ref_md.relative_to(plugin_dir)
                print(f"  - [{ref_title}]({plugin_name}/{rel})")

        print()


if __name__ == "__main__":
    main()
