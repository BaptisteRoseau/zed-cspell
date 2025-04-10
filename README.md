# CSpell Language Server

⚠️⚠️
This plugin is a WIP, if you are looking for the one registered in Zed marketplace it's <https://github.com/mantou132/zed-cspell>

The goal of this plugin is to make it work without having to manually install CSpell. When working, I might push
the changes in [mantou132](https://github.com/mantou132)'s repo since it is the one registered in the Marketplace.
⚠️⚠️

[CSpell Language Server](https://github.com/streetsidesoftware/vscode-spell-checker) support for Zed editor.

This is the engine behind the famous [Code Spell Checker](https://marketplace.visualstudio.com/items?itemName=streetsidesoftware.code-spell-checker) from VSCode.

## Installation

Currently, you need to clone this repository and install this extension using the "Install Dev Extension" button in your editor. Select the whole directory.

This extension relies on `node` and `npm`, which should be installed on your machine beforehand.

Once installed, add execution rights to the script (version might differ):

```bash
chmod 500 ~/.local/share/zed/extensions/work/cspell/cspell-vscode-4.0.13/extension/cspell-lsp
```

## Configuration

The CSpell extension can be configured through a `.cspell.json` configuration file, which reference can be found [here](https://cspell.org/configuration/).

Additionally, you can configure it in your Zed's settings with the following:

```javascript
{
    "lsp": {
        "cspell": {
            "initialization_options": {
                // Your configuration from the reference here
            }
        }
    }
}
```

Since default `Add to dictionary` code action does not work with Zed, a workaround is to define a custom dictionary in your `.cspell.json` configuration file:

```json
{
    "dictionaries": ["custom"],
    "dictionaryDefinitions": [
        { "name": "custom", "path": "./.cspell_dict.txt" }
    ]
}
```

Then, use `Add to custom dictionary` instead.
