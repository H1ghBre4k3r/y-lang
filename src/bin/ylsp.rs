use std::fs;

use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::notification::PublishDiagnostics;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::error;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};
use why_lib::lexer::{self, Span};
use why_lib::parser::{self, ParseError};
use why_lib::typechecker::{self};

#[derive(Debug)]
struct Backend {
    client: Client,
}

impl Backend {
    async fn check_diagnostics(&self, uri: Url) {
        let path = uri.path();
        self.client
            .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                uri: uri.clone(),
                version: None,
                diagnostics: self.get_diagnostics_for_file(path),
            })
            .await;
    }

    fn get_diagnostics_for_file(&self, filename: &str) -> Vec<Diagnostic> {
        let Ok(content) = fs::read_to_string(filename) else {
            return vec![];
        };
        self.get_diagnostics_for_code(&content)
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

        let _ = match typechecker::TypeChecker::new(parsed).check() {
            Ok(checked) => checked,
            Err(e) => {
                let position = e.span();
                let message = e.err().to_string();
                return Some((message, position));
            }
        };

        None
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("ylsp".into()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: true,
                        ..Default::default()
                    },
                )),
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
            text_document: TextDocumentItem { uri, .. },
        } = params;
        self.check_diagnostics(uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
            ..
        } = params;
        self.check_diagnostics(uri).await;
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

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
