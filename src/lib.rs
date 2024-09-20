use std::fs;

use zed_extension_api::{
    self as zed, settings::LspSettings, Architecture, Command, LanguageServerId, Os, Result,
    Worktree,
};

struct CSpellBinary {
    path: String,
    args: Option<Vec<String>>,
}

struct CSpellExtension {
    cached_binary_path: Option<String>,
}

impl CSpellExtension {
    #[allow(dead_code)]
    pub const LANGUAGE_SERVER_ID: &'static str = "cspell";

    fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<CSpellBinary> {
        if let Some(path) = worktree.which("cspell-lsp") {
            return Ok(CSpellBinary {
                path,
                args: Some(vec![]),
            });
        }

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
            "tekumara/CSpell-lsp",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, architecture) = zed::current_platform();
        let version = release.version;

        let asset_name = Self::binary_release_name(&version, &platform, &architecture);
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("CSpell-lsp-{}", version);
        let binary_path = format!("{version_dir}/CSpell-lsp");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let file_kind = match platform {
                zed::Os::Windows => zed::DownloadedFileType::Zip,
                _ => zed::DownloadedFileType::GzipTar,
            };
            zed::download_file(&asset.download_url, &version_dir, file_kind)
                .map_err(|e| format!("failed to download file: {e}"))?;

            Self::clean_other_installations(&version_dir)?;
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(CSpellBinary {
            path: binary_path,
            args: Some(vec![]),
        })
    }

    fn binary_release_name(version: &String, platform: &Os, architecture: &Architecture) -> String {
        format!(
            "CSpell-lsp-{version}-{arch}-{os}.{ext}",
            version = version,
            arch = match architecture {
                Architecture::Aarch64 => "aarch64",
                Architecture::X86 | Architecture::X8664 => "x86_64",
            },
            os = match platform {
                zed::Os::Mac => "apple-darwin",
                zed::Os::Linux => "unknown-linux-gnu",
                zed::Os::Windows => "pc-windows-msvc",
            },
            ext = match platform {
                zed::Os::Windows => "zip",
                _ => "tar.gz",
            }
        )
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
    use zed_extension_api::{Architecture, Os};

    use crate::CSpellExtension;

    #[test]
    fn release_name() {
        assert_eq!(
            CSpellExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Mac,
                &Architecture::Aarch64
            ),
            "CSpell-lsp-v0.1.23-aarch64-apple-darwin.tar.gz".to_string()
        );
        assert_eq!(
            CSpellExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Windows,
                &Architecture::Aarch64
            ),
            "CSpell-lsp-v0.1.23-aarch64-pc-windows-msvc.zip".to_string()
        );
        assert_eq!(
            CSpellExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Linux,
                &Architecture::Aarch64
            ),
            "CSpell-lsp-v0.1.23-aarch64-unknown-linux-gnu.tar.gz".to_string()
        );
        assert_eq!(
            CSpellExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Mac,
                &Architecture::X86
            ),
            "CSpell-lsp-v0.1.23-x86_64-apple-darwin.tar.gz".to_string()
        );
        assert_eq!(
            CSpellExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Windows,
                &Architecture::X86
            ),
            "CSpell-lsp-v0.1.23-x86_64-pc-windows-msvc.zip".to_string()
        );
        assert_eq!(
            CSpellExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Linux,
                &Architecture::X86
            ),
            "CSpell-lsp-v0.1.23-x86_64-unknown-linux-gnu.tar.gz".to_string()
        );
        assert_eq!(
            CSpellExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Mac,
                &Architecture::X8664
            ),
            "CSpell-lsp-v0.1.23-x86_64-apple-darwin.tar.gz".to_string()
        );
        assert_eq!(
            CSpellExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Windows,
                &Architecture::X8664
            ),
            "CSpell-lsp-v0.1.23-x86_64-pc-windows-msvc.zip".to_string()
        );
        assert_eq!(
            CSpellExtension::binary_release_name(
                &"v0.1.23".to_string(),
                &Os::Linux,
                &Architecture::X8664
            ),
            "CSpell-lsp-v0.1.23-x86_64-unknown-linux-gnu.tar.gz".to_string()
        );
    }
}
