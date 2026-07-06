# Themes

You can create custom themes for the browser by creating a TOML file in the themes directory. And then selecting it in the global user preference TOML file.

## Restrictions

- Each theme is limited to **maximum size** of **1 KiB**.
- You can have up to **100 custom themes** in the themes directory. If you exceed this limit, the browser will ignore any additional themes and only load the first 100 themes it finds in the directory.
- Theme **names** must be **unique** and can't be the same as the default themes (light, dark).

## Location

Themes are loaded from the `themes` directory in the user data folder.

### Global Themes

The global themes are available to all profiles and are stored in the `themes` directory in the user data folder.

| OS      | Path                                                     |
| ------- | -------------------------------------------------------- |
| Windows | `%APPDATA%\Browser\Data\themes\`                         |
| Linux   | `$XDG_CONFIG_HOME/.local/share/browser/themes/`          |
| macOS   | `$HOME/Library/Application Support/Browser/Data/themes/` |

### Profile Specific Themes

The profile specific themes are available only to the profile they are stored in and are stored in the `themes` directory in the user data folder for that profile. The default profile is named `default`, but you can create additional profiles by creating a new folder in the user data folder with the name of the profile you want to create.

| OS      | Path                                                               |
| ------- | ------------------------------------------------------------------ |
| Windows | `%APPDATA%\Browser\Data\<PROFILE>\themes\`                         |
| Linux   | `$XDG_CONFIG_HOME/.local/share/browser/<PROFILE>/themes/`          |
| macOS   | `$HOME/Library/Application Support/Browser/Data/<PROFILE>/themes/` |

## Default Themes

The browser comes with two default themes: `light` and `dark`. You can use these themes by setting the `theme` value in the user preferences TOML file to `light` or `dark`. These are the default themes and can't be modified or deleted as they do not exist as actual files in the themes directory.

- [Default Light](./themes/light.toml)
- [Default Dark](./themes/dark.toml)

## Notes

- You can use any installed font that is on your system in your theme.

## See also

- [User Preferences](./preferences.md)
