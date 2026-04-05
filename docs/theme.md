# Themes

You can create custom themes for the browser by creating a TOML file in the themes directory. And then selecting it in the global user preference TOML file.

## Location

Themes are loaded from the `themes` directory in the configuration folder. The location of the configuration folder depends on the operating system:

| OS      | Path                                                       |
| ------- | ---------------------------------------------------------- |
| Windows | `%APPDATA%\Browser\Config\themes\`                         |
| Linux   | `$XDG_CONFIG_HOME/.config/browser/themes/`                 |
| macOS   | `$HOME/Library/Application Support/Browser/Config/themes/` |

## Default Themes

The browser comes with two default themes: `light` and `dark`. You can use these themes by setting the `theme` value in the user preferences TOML file to `light` or `dark`. These are the default themes and can't be modified or deleted as they do not exist as actual files in the themes directory.

- [Default Light](./themes/light.toml)
- [Default Dark](./themes/dark.toml)

## Notes

- Names must be unique and can't be the same as the default themes (light, dark).
- You can use any system avaliable font in the theme configuration.
