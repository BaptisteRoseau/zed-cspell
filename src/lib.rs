use std::process::Command as ProcCommand;
use std::{fmt::format, fs};

use zed_extension_api::{
    self as zed, settings::LspSettings, Command, LanguageServerId, Result, Worktree,
};

struct CSpellBinary {
    path: String,
    args: Option<Vec<String>>,
}

struct CSpellExtension {
    cached_binary_path: Option<String>,
}

//FIXME:
// - The script is not executable: give it 700 permission, but unix cannot be compiled through Zed
// - The "Add to dictionnary" does not work because it relies on VSCode's configuration path
// - Install **only** the CSpell dictionnaries from NPM

// Honestly, our best bet is might be to rewrite this plugin is Rust:

// TODO: Make an executable file as stated in https://medium.com/@zetavg/howto-port-the-vscode-code-spell-checker-cspell-plugin-to-sublime-6a7f71fad462
// Make a script and run it as the binary name
// Take care of the OS version though
// Run it using "node <path to bin>" and specify where the node_modules are ~/.local/share/zed/extensions/work/cspell/cspell-vscode-4.0.13/extension
// Or run it directly from ~/.local/share/zed/extensions/work/cspell/cspell-vscode-4.0.13/extension
// Then make sure all the actions/config work properly

// For languages, install the node_module corresponding the the languagem: ex @cspell/dict-fr-fr
// Make a command with the available languages ? Like "Enable French"
// Or add them to node_modules/@cspell/cspell-bundled-dicts/cspell-default.config.js ?

impl CSpellExtension {
    #[allow(dead_code)]
    pub const LANGUAGE_SERVER_ID: &'static str = "cspell";

    fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<CSpellBinary> {
        let _ = worktree;

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(CSpellBinary {
                    path: path.clone(),
                    args: Some(vec![]),
                });
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "streetsidesoftware/vscode-spell-checker",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let version = release.version;
        let version_number = version
            .split('v')
            .last()
            .ok_or("Invalid binary name")?
            .to_string();

        let asset_name = Self::binary_release_name(&version_number);
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("cspell-vscode-{}", version_number);
        let main_cjs = format!("{version_dir}/extension/packages/_server/dist/main.cjs");

        if !fs::metadata(&main_cjs).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::Zip,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            Self::clean_other_installations(&version_dir)?;
            Self::install_node_modules(&version_dir)?;
        }

        let binary_path = Self::make_script_linux(version_dir.as_str())?;

        self.cached_binary_path = Some(binary_path.clone());
        Ok(CSpellBinary {
            path: binary_path,
            args: Some(vec![]),
        })
    }

    fn binary_release_name(version: &String) -> String {
        format!("code-spell-checker-{version}.vsix", version = version)
    }

    fn clean_other_installations(version_to_keep: &String) -> Result<(), String> {
        let entries =
            fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
            if entry.file_name().to_str() != Some(version_to_keep) {
                fs::remove_dir_all(entry.path()).ok();
            }
        }
        Ok(())
    }

    /// Make a script because we need to run the command:
    ///
    ///     node <extension_install_folder>/extension/packages/_server/dist/main.cjs --stdio
    ///
    /// But Zed extension expect an executable relative to the install folder.
    fn make_script_linux(version_dir: &str) -> Result<String, String> {
        let content = r#"#!/usr/bin/env bash
            SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
            node "$SCRIPT_DIR/packages/_server/dist/main.cjs" --stdio "$@""#
            .to_string();
        let script_path = format!("{}/extension/cspell-lsp", version_dir);
        fs::write(&script_path, content)
            .map_err(|e| format!("failed to write script file: {e}"))?;
        // let mut permissions = std::fs::File::open(script_path)
        //     .and_then(|f| f.metadata()).map(|m| m.permissions())
        //     .map_err(|op| format!("Error while setting up script permissions {op}"))?;
        // permissions.set_mode(0o700);

        Ok(script_path)
    }

    // TODO: install ONLY the @cspell dictionaries
    fn install_node_modules(version_dir: &str) -> Result<(), String> {
        ProcCommand::new("npm")
            .arg("install")
            .current_dir(format!("{version_dir}/extension"))
            .output()
            .map_err(|e| format!("failed to install CSpell node modules: {e}"))?;
        Ok(())
    }
}

impl zed::Extension for CSpellExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let cspell_binary = self.language_server_binary(language_server_id, worktree)?;

        Ok(zed::Command {
            command: cspell_binary.path,
            args: cspell_binary.args.unwrap(),
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.initialization_options.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }
}

zed::register_extension!(CSpellExtension);

#[cfg(test)]
mod tests {
    use crate::CSpellExtension;
}
