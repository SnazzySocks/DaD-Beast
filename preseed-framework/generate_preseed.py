#!/usr/bin/env python3
"""Generate a preseed.cfg from a template and JSON configuration."""

import json
from pathlib import Path

CONFIG_PATH = Path(__file__).with_name("config.json")
TEMPLATE_PATH = Path(__file__).with_name("template.cfg")
OUTPUT_PATH = Path(__file__).with_name("preseed.cfg")


def main():
    if not CONFIG_PATH.exists():
        raise SystemExit(f"Config file {CONFIG_PATH} not found")
    if not TEMPLATE_PATH.exists():
        raise SystemExit(f"Template file {TEMPLATE_PATH} not found")

    with CONFIG_PATH.open() as f:
        config = json.load(f)

    with TEMPLATE_PATH.open() as f:
        content = f.read()

    for key, value in config.items():
        placeholder = f"{{{{{key}}}}}"
        content = content.replace(placeholder, str(value))

    OUTPUT_PATH.write_text(content)
    print(f"Generated {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
