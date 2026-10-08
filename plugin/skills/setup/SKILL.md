---
name: setup
description: Install the ccline binary and set it as the Claude Code status line in ~/.claude/settings.json
disable-model-invocation: true
---

Set up ccline as the user's status line:

1. Install (or update) the binary by running:

   ```bash
   CLAUDE_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT}" CLAUDE_PLUGIN_DATA="${CLAUDE_PLUGIN_DATA}" sh "${CLAUDE_PLUGIN_ROOT}/scripts/install.sh"
   ```

   The last line of output is the absolute path of the installed binary. If the script fails, show the error to the user and stop.

2. Read `~/.claude/settings.json` (treat a missing file as `{}`). If it already has a `statusLine` whose `command` is not this binary path, show the user the current value and confirm before replacing it.

3. Set the `statusLine` key, keeping every other key in the file unchanged:

   ```json
   {
     "statusLine": {
       "type": "command",
       "command": "<binary path from step 1>"
     }
   }
   ```

4. Tell the user ccline is now their status line and appears on the next refresh. The plugin keeps the binary at that path up to date when the plugin updates, so the setting doesn't need to change again. To remove it, delete the `statusLine` key and uninstall the plugin.
