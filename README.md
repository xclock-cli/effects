# xclock effects

External effects for [xclock](https://github.com/xclock-cli/xclock).

An effect is an executable that follows the plugin protocol and responds with
animation `frames` instead of static lines. The kernel renders the frames
inside any `plugin` widget, which lets you animate dashboards without touching
the core.

## Effects

| Effect | Description | Args |
|--------|-------------|------|
| `decrypt` | Reveals text through random glyphs | `lines`, `delay_ms`, `seed` |
| `typewriter` | Reveals text character by character | `lines`, `delay_ms` |

## Use

```jsonc
{
  "layout": {
    "direction": "horizontal",
    "ratio": [50, 50],
    "children": [
      { "widget": "clock" },
      {
        "widget": "plugin",
        "options": {
          "name": "decrypt",
          "args": {
            "lines": ["focus time", "20:00 left"],
            "delay_ms": 60
          }
        }
      }
    ]
  }
}
```

Install the compiled effect into the xclock plugins directory and reference it
by name. `xclock plugin list` shows everything available.

## Development

```bash
cargo build --release
cargo test
```

`effects-lib` contains the shared frame helpers used by the official effects.
The crates depend on the
[`xclock-plugin-api`](https://github.com/xclock-cli/api) crate from its
repository.
