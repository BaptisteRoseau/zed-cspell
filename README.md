# CSpell Language Server

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
