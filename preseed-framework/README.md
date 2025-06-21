# Debian Preseed Framework

This framework provides a simple starting point for creating automated installation
media for Debian-based distributions using a preseed file.

## Files

- `template.cfg` – A preseed template containing placeholders for various options.
- `config.json` – A configuration file with the values that will be substituted
  into the template.
- `generate_preseed.py` – A helper script that reads `config.json` and outputs a
  completed `preseed.cfg`.

## Usage

1. Edit `config.json` to match your desired configuration.
2. Run `python3 generate_preseed.py` to generate `preseed.cfg`.
3. Place the generated `preseed.cfg` on your installation media or provide it via
   the boot parameters when starting the installer.

The generated file can be used with any Debian-based installer that supports
preseed, such as Debian, Ubuntu, or Linux Mint.
