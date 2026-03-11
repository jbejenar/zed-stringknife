use zed_extension_api::{self as zed, LanguageServerId, Result};

struct StringKnifeExtension;

impl zed::Extension for StringKnifeExtension {
    fn new() -> Self {
        StringKnifeExtension
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // In dev mode, use the locally built binary.
        // For published releases, the binary is downloaded from GitHub Releases.
        let path = worktree
            .which("stringknife-lsp")
            .ok_or_else(|| "stringknife-lsp binary not found in PATH".to_string())?;

        Ok(zed::Command {
            command: path,
            args: vec!["--stdio".to_string()],
            env: Vec::new(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        Ok(None)
    }
}

zed::register_extension!(StringKnifeExtension);
