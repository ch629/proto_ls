use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    /// The process Id of the parent process that started the server. Is null if
    /// the process has not been started by another process. If the parent
    /// process is not alive then the server should exit (see exit notification)
    /// its process.
    processor_id: Option<isize>,

    /// Information about the client
    client_info: Option<ClientInfo>,

    /// The locale the client is currently showing the user interface
    /// in. This must not necessarily be the locale of the operating
    /// system.
    ///
    /// Uses IETF language tags as the value's syntax
    /// (See https://en.wikipedia.org/wiki/IETF_language_tag)
    locale: Option<String>,

    /// The rootPath of the workspace. Is null
    /// if no folder is open.
    ///
    /// @deprecated in favour of `rootUri`.
    root_path: Option<String>,

    /// The rootUri of the workspace. Is null if no
    /// folder is open. If both `rootPath` and `rootUri` are set
    /// `rootUri` wins.
    ///
    /// @deprecated in favour of `workspaceFolders`
    root_uri: Option<String>,

    /// User provided initialization options.
    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#lspAny
    initialize_options: Option<LspAny>,

    /// The capabilities provided by the client (editor or tool)
    capabilities: ClientCapabilities,

    /// The initial trace setting. If omitted trace is disabled ('off').
    trace: Option<TraceValue>,

    /// The workspace folders configured in the client when the server starts.
    /// This property is only available if the client supports workspace folders.
    /// It can be `null` if the client supports workspace folders but none are
    /// configured.
    workspace_folders: Option<Vec<WorkspaceFolder>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientInfo {
    /// The name of the client as defined by the client.
    name: String,

    /// The client's version as defined by the client.
    version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkspaceFolder {
    /// The associated URI for this workspace folder.
    uri: String,

    /// The name of the workspace folder. Used to refer to this
    /// workspace folder in the user interface.
    name: String,
}

/// oneof off, messages, verbose
#[derive(Debug, Deserialize, Serialize)]
pub enum TraceValue {
    Off,
    Messages,
    Verbose,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ClientCapabilities {
    /// Workspace specific client capabilities.
    workspace: Option<WorkspaceClientCapabilities>,

    /// Text document specific client capabilities.
    text_document: Option<TextDocumentClientCapabilities>,

    /// Capabilities specific to the notebook document support.
    notebook_document: Option<NodebookDocumentClientCapabilities>,

    /// Window specific client capabilities.
    window: Option<WindowClientCapabilities>,

    /// General client capabilities.
    general: Option<GeneralClientCapabilities>,

    /// Experimental client capabilities.
    experimental: Option<LspAny>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkspaceClientCapabilities {
    /// The client supports applying batch edits
    /// to the workspace by supporting the request
    /// 'workspace/applyEdit'
    apply_edit: Option<bool>,

    /// Capabilities specific to `WorkspaceEdit`s
    workspace_edit: Option<WorkspaceEditClientCapabilities>,

    /// Capabilities specific to the `workspace/didChangeConfiguration`
    /// notification.
    did_change_configuration: Option<DidChangeConfigurationClientCapabilities>,

    /// Capabilities specific to the `workspace/didChangeWatchedFiles`
    /// notification.
    did_change_watched_files: Option<DidChangeWatchedFilesClientCapabilities>,

    /// Capabilities specific to the `workspace/symbol` request.
    symbol: Option<SymbolWorkspaceClientCapabilities>,

    /// Capabilities specific to the `workspace/executeCommand` request.
    execute_command: Option<ExecuteCommandClientCapabilities>,

    /// The client has support for workspace folders.
    workspace_folders: Option<bool>,

    /// The client supports `workspace/configuration` requests.
    configuration: Option<bool>,

    /// Capabilities specific to the semantic token requests scoped to the
    /// workspace.
    semantic_tokens: Option<SemanticTokensWorkspaceClientCapabilities>,

    /// Capabilities specific to the code lens requests scoped to the
    /// workspace.
    code_lens: Option<CodeLensWorkspaceClientCapabilities>,

    /// The client has support for file requests/notifications.
    file_operations: Option<FileWorkspaceOperations>,

    /// Client workspace capabilities specific to inline values.
    inline_value: Option<InlineValueWorkspaceClientCapabilities>,

    /// Client workspace capabilities specific to inlay hints.
    inlay_hint: Option<InlayHintWorkspaceClientCapabilities>,

    /// Client workspace capabilities specific to diagnostics.
    diagnostics: Option<DiagnosticsWorkspaceClientCapabilities>,
}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct WorkspaceEditClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct DidChangeConfigurationClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct DidChangeWatchedFilesClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct SymbolWorkspaceClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct ExecuteCommandClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct CodeLensWorkspaceClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct SemanticTokensWorkspaceClientCapabilities {}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileWorkspaceOperations {
    /// Whether the client supports dynamic registration for file
    /// requests/notifications.
    dynamic_registration: Option<bool>,

    /// The client has support for sending didCreateFiles notifications.
    did_create: Option<bool>,

    /// The client has support for sending willCreateFiles requests.
    will_create: Option<bool>,

    /// The client has support for sending didRenameFiles notifications.
    did_rename: Option<bool>,

    /// The client has support for sending willRenameFiles requests.
    will_rename: Option<bool>,

    /// The client has support for sending didDeleteFiles notifications.
    did_delete: Option<bool>,

    /// The client has support for sending willDeleteFiles requests.
    will_delete: Option<bool>,
}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct InlineValueWorkspaceClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct InlayHintWorkspaceClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct DiagnosticsWorkspaceClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct TextDocumentClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct NodebookDocumentClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct WindowClientCapabilities {}

// TODO:
#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralClientCapabilities {}

// TODO: Any should be any LSP type (Object, String, Int, )
#[derive(Debug, Deserialize, Serialize)]
pub enum LspAny {}
