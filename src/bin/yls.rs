use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;

use tower_lsp_server::jsonrpc::{Error, Result};
use tower_lsp_server::lsp_types::notification::PublishDiagnostics;
use tower_lsp_server::lsp_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};
use tracing::error;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};
use why_lib::formatter;
use why_lib::lexer::{self, Span};
use why_lib::parser::{self, ParseError};
use why_lib::typechecker::{self};

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<RwLock<HashMap<Uri, String>>>,
}

impl Backend {
    async fn check_diagnostics(&self, uri: Uri) {
        let path = uri.path().as_str();
        if !path.ends_with(".why") {
            return;
        }

        // Try to get content from document store first, fall back to file system
        let content = {
            let documents = self.documents.read().await;
            if let Some(content) = documents.get(&uri) {
                content.clone()
            } else {
                match fs::read_to_string(path) {
                    Ok(content) => content,
                    Err(_) => return,
                }
            }
        };

        self.client
            .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                uri: uri.clone(),
                version: None,
                diagnostics: self.get_diagnostics_for_code(&content),
            })
            .await;
    }

    fn get_diagnostics_for_code(&self, input: &str) -> Vec<Diagnostic> {
        let Some((message, pos)) = self.perform_code_analysis(input) else {
            return vec![];
        };

        let Span { start, end, .. } = pos;

        vec![Diagnostic {
            range: Range {
                start: Position {
                    line: start.0 as u32,
                    character: start.1 as u32,
                },
                end: Position {
                    line: end.0 as u32,
                    character: end.1 as u32,
                },
            },
            message,
            ..Default::default()
        }]
    }

    fn perform_code_analysis(&self, input: &str) -> Option<(String, Span)> {
        let lexed = match lexer::Lexer::new(input).lex() {
            Ok(lexed) => lexed,
            Err(e) => {
                error!("LexError: {e}");
                return None;
            }
        };

        let parsed = match parser::parse(&mut lexed.into()) {
            Ok(parsed) => parsed,
            Err(e) => {
                error!("{e}");
                let ParseError { message, position } = e;
                let position = position.unwrap_or(Span::default());
                return Some((message, position));
            }
        };

        let checked = match typechecker::TypeChecker::new(parsed).check() {
            Ok(checked) => checked,
            Err(e) => {
                let position = e.span();
                let message = e.err().to_string();
                return Some((message, position));
            }
        };

        let _ = match typechecker::TypeChecker::validate(checked) {
            Ok(validated) => validated,
            Err(e) => {
                let position = e.span();
                let message = e.err();
                return Some((message, position));
            }
        };

        None
    }

    fn format_code(&self, input: &str) -> std::result::Result<String, String> {
        // Parse the input
        let lexed = match lexer::Lexer::new(input).lex() {
            Ok(lexed) => lexed,
            Err(e) => {
                return Err(format!("Lexer error: {}", e));
            }
        };

        let parsed = match parser::parse(&mut lexed.into()) {
            Ok(parsed) => parsed,
            Err(e) => {
                return Err(format!("Parse error: {}", e));
            }
        };

        // Format the AST
        match formatter::format_program(&parsed) {
            Ok(formatted) => Ok(formatted),
            Err(e) => Err(format!("Formatting error: {}", e)),
        }
    }

    fn get_document_end_position(&self, content: &str) -> Position {
        if content.is_empty() {
            return Position::new(0, 0);
        }

        // Count lines properly - use lines() iterator which handles line endings correctly
        let lines: Vec<&str> = content.lines().collect();
        let line_count = lines.len() as u32;
        
        // If content ends with a newline, we have an extra empty line
        let actual_line_count = if content.ends_with('\n') {
            line_count
        } else {
            line_count.saturating_sub(1)
        };
        
        // Get the character position of the last line
        let last_line_length = if let Some(last_line) = lines.last() {
            last_line.len() as u32
        } else {
            0
        };

        Position::new(actual_line_count, last_line_length)
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                // diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                //     DiagnosticOptions {
                //         identifier: Some("ylsp".into()),
                //         inter_file_dependencies: true,
                //         workspace_diagnostics: true,
                //         ..Default::default()
                //     },
                // )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: None,
                    file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                        did_create: Some(FileOperationRegistrationOptions {
                            filters: vec![FileOperationFilter {
                                scheme: None,
                                pattern: FileOperationPattern {
                                    glob: "**/*.why".into(),
                                    ..Default::default()
                                },
                            }],
                        }),
                        ..Default::default()
                    }),
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::ERROR, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::ERROR, "server shutdown!")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let DidOpenTextDocumentParams {
            text_document: TextDocumentItem { uri, text, .. },
        } = params;
        
        // Store the document content
        {
            let mut documents = self.documents.write().await;
            documents.insert(uri.clone(), text);
        }
        
        self.check_diagnostics(uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
            text,
            ..
        } = params;
        
        // Update document content if provided
        if let Some(text) = text {
            let mut documents = self.documents.write().await;
            documents.insert(uri.clone(), text);
        }
        
        self.check_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, .. },
            content_changes,
        } = params;

        // Apply content changes to our document store
        if let Some(change) = content_changes.into_iter().next() {
            // For FULL sync, we replace the entire content
            if change.range.is_none() {
                let mut documents = self.documents.write().await;
                documents.insert(uri.clone(), change.text);
            }
        }

        self.check_diagnostics(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
        } = params;

        // Remove document from store
        {
            let mut documents = self.documents.write().await;
            documents.remove(&uri);
        }
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        error!("CREATED: {params:?}")
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let _ = params;
        error!("Got a textDocument/diagnostic request, but it is not implemented");
        Err(Error::method_not_found())
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let path = uri.path().as_str();

        // Only format .why files
        if !path.ends_with(".why") {
            return Ok(None);
        }

        // Get content from document store first, fall back to file system
        let content = {
            let documents = self.documents.read().await;
            if let Some(content) = documents.get(&uri) {
                content.clone()
            } else {
                match fs::read_to_string(path) {
                    Ok(content) => content,
                    Err(e) => {
                        error!("Failed to read file {}: {}", path, e);
                        return Ok(None);
                    }
                }
            }
        };

        // Format the code
        match self.format_code(&content) {
            Ok(formatted) => {
                if formatted == content {
                    // No changes needed
                    return Ok(None);
                }

                // Create a TextEdit that replaces the entire document
                let text_edit = TextEdit {
                    range: Range {
                        start: Position::new(0, 0),
                        // Calculate the end position of the original document
                        end: self.get_document_end_position(&content),
                    },
                    new_text: formatted,
                };

                Ok(Some(vec![text_edit]))
            }
            Err(e) => {
                error!("Failed to format code: {}", e);
                Ok(None)
            }
        }
    }
}

fn init() {
    let filter = filter::Targets::new().with_target("ylsp", tracing::metadata::LevelFilter::TRACE);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .compact()
                .with_ansi(false),
        )
        .with(filter)
        .init();
}

#[tokio::main]
async fn main() {
    init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { 
        client,
        documents: Arc::new(RwLock::new(HashMap::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
