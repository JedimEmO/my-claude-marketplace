# my-claude-marketplace

[**Browse the plugin reference book**](https://jedimemo.github.io/my-claude-marketplace/)

Personal Claude Code plugin marketplace — MCP servers, skills, and tools.

## Usage

Add this marketplace:

```
/plugin marketplace add https://github.com/JedimEmO/my-claude-marketplace.git
```

Install a plugin:

```
/plugin install <plugin-name>@my-claude-marketplace
```

## Adding a plugin

Create a plugin directory under `plugins/`:

```
plugins/my-plugin/
├── .claude-plugin/
│   └── plugin.json
└── skills/            # and/or commands/, agents/, hooks/, .mcp.json
    └── my-skill/
        └── SKILL.md
```

Then add an entry to `.claude-plugin/marketplace.json`:

```json
{
  "name": "my-plugin",
  "source": "my-plugin",
  "description": "What it does",
  "version": "1.0.0"
}
```

The `source` is relative to `plugins/` (configured via `pluginRoot`).
