# Control protocol

Start the app with `--control <port>` (or `DESIGNCRAFT_CONTROL_PORT`). The server listens on
`127.0.0.1` only and speaks JSON lines: one request object per line, one reply per line.

For an isolated native test profile, set `DESIGNCRAFT_CONFIG_DIR` to a dedicated directory before
launch. UI/engine preferences and the `Recovery` directory then live under that root. An unset
override preserves the usual platform paths; an explicitly empty override disables those paths.
`DESIGNCRAFT_NO_PREFS` still disables preference reads/writes independently of recovery.

Control pointer and key input is queued across native frames. After a drag, wait for both the
expected document state and `document.inspect.canUndo` before testing undo; an object can appear
in the preview before its undo entry is committed. On macOS, default shortcuts owned by the native
menu (including Command-Z) are skipped by the egui shortcut handler. `ui.key` does not send a Cocoa
menu event; use `ui.menu.invoke` for a menu-command test and test physical accelerators separately.

**Only requests are read.** Every line must be a JSON object with a string `method` (`id` and `params`
are optional; blank lines are skipped). Anything else gets one error reply
(`{"ok": false, "error": "… closing the connection"}`) and the server **closes the connection**, so
nothing sent after it on that connection runs. That covers text that isn't JSON, a JSON array or number,
an object without `method`, invalid UTF-8, and a line longer than 4 MiB. An HTTP request (for example a
web page's cross-origin `fetch` to `127.0.0.1:<port>`) therefore can't smuggle a command in its body:
its request line is rejected first. At most 16 connections are served at once; further ones get an error
line and are closed. Clients that get an error reply should reconnect. Transport:
`apps/designcraft/src/control_server.rs`.

## Authentication

Every connection must authenticate before anything else runs. Start the app with a token file, so the
token is not visible in the process list:

```sh
designcraft --control 7979 --control-token-file /private/path/designcraft-control.token
```

If the file does not exist, DesignCraft creates it with a fresh 256-bit token (64 hexadecimal
characters); on Unix the new file is mode `0600`, on Windows protect it with a user-only ACL. An
existing file is reused. `--control-token <64 hex>` passes the token directly instead. With neither,
the app makes a token for this launch and prints it on standard error. `DESIGNCRAFT_CONTROL_TOKEN_FILE`
and `DESIGNCRAFT_CONTROL_TOKEN` are the environment equivalents. If the token can't be set up (a
corrupt token file, both options given) the control channel stays off and the reason is logged.

The first request on each connection must be:

```json
{"id": "auth", "method": "auth", "params": {"token": "<64 hexadecimal characters>"}}
{"id": "auth", "ok": true, "result": {"authenticated": true}}
```

Any other first request, or a wrong token, gets `{"ok": false, "error": "authentication required"}`
and the connection is closed; nothing runs. A second `auth` on an authenticated connection gets
`{"ok": false, "error": "already authenticated"}` and the connection keeps serving.
A connection that has not authenticated within 10 seconds is closed, so idle connections cannot hold the 16 connection slots; once authenticated it may stay idle.
`designcraft-cli app`, `script --connect` and `mcp --connect` send it for you: give them the same
`--control-token-file PATH` (or `--control-token`, or the environment variables). The token is a
bearer credential for the whole control surface, which can open and write files: keep the file
private and don't log or commit it.

## File access

Control requests read and write files only beneath two launch-time roots, as in PhotoCraft:

```sh
designcraft --control 7979 --control-token-file /private/path/designcraft-control.token \
  --automation-read-root /work/project --automation-write-root /work/project
```

`DESIGNCRAFT_AUTOMATION_READ_ROOT` and `DESIGNCRAFT_AUTOMATION_WRITE_ROOT` are the environment
equivalents. Each root must be an existing folder; if one can't be opened the control channel stays
off and the reason is logged. **Without roots, every file path in a control request is refused.**
The read root covers what requests open or import (`app.open`, `file.open`, `file.place`,
`data.source.select`, `swatch.load`, …), the write root what they save or export (`app.save`,
`app.export`, `ui.screenshot`, `ui.render`, `file.save*`, `file.export*`, `snippet.export`,
`swatch.save`, …). A command whose direction isn't known needs its path beneath both roots.

A path (`path`, and `dir` except for `text.*` commands, in a request or in the parameters of any
command it runs) is a `/`-separated path relative to the root, or an absolute path inside it. Any other
parameter string that looks absolute (`/…`, `\…`, `C:\…`, `~/…`, `file:…`), at any depth, is treated
as a path too, except free-text fields such as `text`, `name` and `title`. Refused
before any file is touched: `..`, `.` and empty components, `\`, drive or stream prefixes (`:`), Windows
device names, absolute paths outside the root, and paths that leave the root through a link (a link
that doesn't resolve is refused too). The command then gets the resolved absolute path.
`app.save`/`file.save` without `path` still saves back to the document's own file.

Refused outright, because they read files a document, data source or book refers to rather than one the
request names: `book.*`, `file.package`, `links.copyTo`, `links.update`, `data.source.update`, and
`links.relinkFolder` without `dir`. Files a document opened by `app.open` links to, and the images of a
data merge, are still read where the document says; open only documents you trust.

The same rules apply to menu commands, shortcuts and dialog buttons that control input (`ui.key`,
`ui.click`, `ui.dialog.confirm`, …) sets off. Interactive use is not confined, and a file chosen in a
native file dialog during control input must also lie beneath the roots. The check and the file
operation are two steps, so another local process that swaps a folder for a link in between is not
stopped. Implementation: `crates/engine/src/automation.rs`.

```json
{"id": 1, "method": "engine.execute", "params": {"command": "frame.create", "params": {"rect": [36, 36, 300, 200], "content": "text"}}}
{"id": 1, "ok": true, "result": {"id": 10, "story": 11}}
```

| Method | Params | What it does |
|---|---|---|
| `engine.execute` / `ui.menu.invoke` | `{command, params}` | Run any engine or UI command (see `engine.commands`). `ui.menu.invoke` without `params` acts like choosing the menu item: a command labelled "…" that takes parameters opens its dialog |
| `engine.commands` | — | Every command: id, label, menu path, shortcut, params doc, enabled |
| `document.inspect` | — | Pages, spreads, items, stories (overset), styles, swatches, selection |
| `ui.inspect` | — | Tool, UI state, view (zoom/origin), canvas rect, perf |
| `ui.menu.list` / `ui.tool.list` | — | Menu tree / Tools panel groups |
| `ui.tool.select` | `{tool}` | Select a tool (`selection`, `type`, `rectangleFrame`, …) |
| `ui.pointer` | `{events:[{kind: down\|drag\|up\|move\|doubleclick, x, y, space?: "screen"\|"canvas"}], mods?}` | Drive the active tool through the same code path as the mouse |
| `ui.key` / `ui.text` | `{key, shift?, alt?, cmd?}` / `{text}` | Synthetic keyboard input (typing into a text frame) |
| `ui.move` / `ui.click` / `ui.drag` | screen points, `button?: left\|right\|middle` | Real egui pointer input — reaches every widget, menu and panel |
| `ui.set` | `{brightness?, panel?, rulers?, guides?, frameEdges?, baselineGrid?, textThreads?, screenMode?, zoom?, page?, fit?}` | UI state |
| `ui.dialog.open` | `{id, fields?}` | Open a dialog by id (e.g. `paragraphStyleOptions` or `characterStyleOptions` with `{name, section}`, or `{new: true}` for a new style) |
| `ui.dialog.set` / `ui.dialog.confirm` / `ui.dialog.cancel` | `{field, value}` | Fill and confirm the open dialog |
| `ui.resize`, `ui.focus` | | Window control |
| `ui.screenshot` | `{path?}` | PNG of the whole window: written to `path` beneath the write root, or returned as `pngBase64` |
| `ui.render` | `{path?, page?, scale?, bleed?}` | Render a page headlessly: `pngBase64`, also written to `path` (beneath the write root) when given |
| `app.open` / `app.save` / `app.export` | `{path}` / `{path?}` / `{path, page?, scale?}` | Files, beneath the read / write roots ([File access](#file-access)) |
| `app.quit` | | Quit without asking about unsaved changes (the `app.quit` command, like ⌘Q, asks) |

Data merge runs through `engine.execute` like any command: `data.source.select`, `data.fields`, `data.placeholder.add` /
`.remove`, `data.options`, `data.preview` / `data.preview.stop`, `data.merge`. `data.merge` creates and activates a new
merged document; the template stays as it was ([agents.md](agents.md#data-merge)).

Headless window screenshots (locked screen, hidden window): `cargo run -p designcraft-ui-egui --example ui_shot -- script.jsonl`, where each line is one of the requests above, `{"shot": "/abs/out.png"}` or `{"steps": n}` (renders the whole UI offscreen with wgpu).

The MCP server (`designcraft-cli mcp`) wraps the same methods for Claude and other agents.

## Shared panel docking

Panel arrangements support tab grouping and reordering, horizontal and vertical splits,
and movable, resizable floating groups inside the application window. Drag a panel tab,
or use its context menu. Floating groups remain within the application viewport.

The shared layout begins with the existing workspace on the first docking operation.
Workspace saves retain its groups, active tabs, split sizes, floating geometry, and the
positions of hidden panels. Resetting a built-in workspace restores its default dock.

- `window.panel.move {panel, anchor, zone?, before?}` moves a panel into the target group.
  `zone` is `tab` (default), `center`, `left`, `right`, `top`, or `bottom`; `before` reorders
  tabs by panel ID. The destination must already be visible.
- `window.panel.float {panel, x?, y?, width?, height?}` detaches a panel.
- `window.panel.dock {panel, anchor?, zone?, before?}` returns a panel to its previous
  dock placement when possible, or to the supplied destination.
- `window.panel.activate {panel}` reveals and selects a panel, restoring its saved
  position when possible. `window.panel.close {panel}` hides it.

Panel IDs are stable across themes. Invalid IDs, non-finite or invalid geometry, and
invalid destinations return an error without changing the arrangement.

`window.panel.layout {action}` also accepts the shared, externally tagged action schema.
For example, `{"action":{"MoveFloating":{"panel":"properties","rect":[44,66,360,450]}}}`
updates a floating group's position and size; `{"action":{"ResizeSplit":{"path":[],"size":{"Ratio":0.35}}}}`
resizes the root divider (`false`/`true` path entries select first/second children).
`Move` with `placement: {"Tab":{"before":"swatches"}}` reorders tabs, and
`SetStackOpen {panel,open}` / `ResizeStack {panel,height}` control accordion entries.
The UI dispatches these same commands. Unknown panel IDs, invalid geometry, and stale
split paths return an error without changing the workspace.

After panel customization, `app.tablePanel` (including Shift+F9) still toggles the
Table panel. `ui.set` with `closePanel` closes the last explicitly opened panel;
`dockExpanded:false` hides the root dock while floating groups remain visible.
The icon rail can reopen a collapsed dock through any root panel. Activating a
floating panel also raises its group. These operations change only UI layout.
