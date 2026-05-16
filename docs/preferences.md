# User Preferences

**Location**

The user preferences are stored in a `preferences.toml` file in the configuration folder.

The location of the configuration folder depends on the operating system:

| OS      | Path                                                                |
| ------- | ------------------------------------------------------------------- |
| Windows | `%APPDATA%\Browser\Config\preferences.toml`                         |
| Linux   | `$XDG_CONFIG_HOME/.config/browser/preferences.toml`                 |
| macOS   | `$HOME/Library/Application Support/Browser/Config/preferences.toml` |

## Values

### Theme (String)

The `theme` value specifies the theme to use for the browser. The default value is `light`. You can set it to `dark` to use the dark theme, or the name of a custom theme that you have created.

**Example:**

```toml
theme = "dark"
```

### Force Dark (Boolean)

The `force_dark` value specifies whether to force dark mode for all websites. The default value is `false`. If set to `true`, the browser will attempt to render all websites in dark mode, even if they do not support it. If you also set the runtime argument `--force-dark` to `true`, the browser will force dark mode for all websites regardless of the value of this preference.

**Example:**

```toml
force_dark = true
```

## See also

- [Themes](./theme.md)
