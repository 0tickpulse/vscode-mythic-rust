mod documents;
mod errors;
mod mythic_parser;
mod utilities;
mod yaml;
use core::{marker::Send, pin::Pin};
use std::mem::take;

use chumsky::primitive::Container;
use dashmap::DashMap;
use documents::{DocumentInfo, LEGEND_TYPE};
use errors::error_registry::Error;
use ropey::Rope;
use tokio::{io::AsyncWriteExt, join};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, DocumentFilter, InitializeParams,
        InitializeResult, InitializedParams, MessageType, SemanticToken, SemanticTokens,
        SemanticTokensClientCapabilities, SemanticTokensFullOptions, SemanticTokensLegend,
        SemanticTokensOptions, SemanticTokensParams, SemanticTokensRegistrationOptions,
        SemanticTokensResult, SemanticTokensServerCapabilities, ServerCapabilities,
        StaticRegistrationOptions, TextDocumentItem, TextDocumentRegistrationOptions,
        TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
    },
    Client, LanguageServer, LspService, Server,
};
use utilities::positions_and_ranges::{CustomPosition, CustomRange};
use yaml_rust::YamlLoader;

#[derive(Debug)]
pub struct Backend {
    /// The client that we will send notifications to.
    client: Client,
    /// A map of cached document information.
    document_map: DashMap<String, DocumentInfo>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                position_encoding: None,
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                selection_range_provider: None,
                hover_provider: None,
                completion_provider: None,
                signature_help_provider: None,
                definition_provider: None,
                type_definition_provider: None,
                implementation_provider: None,
                references_provider: None,
                document_highlight_provider: None,
                document_symbol_provider: None,
                workspace_symbol_provider: None,
                code_action_provider: None,
                code_lens_provider: None,
                document_formatting_provider: None,
                document_range_formatting_provider: None,
                document_on_type_formatting_provider: None,
                rename_provider: None,
                document_link_provider: None,
                color_provider: None,
                folding_range_provider: None,
                declaration_provider: None,
                execute_command_provider: None,
                workspace: None,
                call_hierarchy_provider: None,
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("MythicYAML".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: LEGEND_TYPE.into(),
                                    token_modifiers: vec![],
                                },
                                range: Some(false),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),
                moniker_provider: None,
                inline_value_provider: None,
                inlay_hint_provider: None,
                linked_editing_range_provider: None,
                experimental: None,
            },
            offset_encoding: None,
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "shutting down!")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(params.text_document).await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: take(&mut params.content_changes[0].text),
            version: params.text_document.version,
            language_id: String::from("yaml"),
        })
        .await
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        self.client
            .log_message(MessageType::INFO, "semantic tokens!")
            .await;
        let doc_info = self.document_map.get(&params.text_document.uri.to_string());

        if doc_info.is_none() {
            return Ok(None);
        }
        let doc_info = doc_info.unwrap();

        let doc_info = doc_info.value();
        let tokens = &mut doc_info.semantic_tokens.clone();

        let semantic_tokens = (|| {
            let rope = &doc_info.source;
            tokens.sort_by(|a, b| a.start.cmp(&b.start));
            let mut pre_line = 0;
            let mut pre_start = 0;
            let semantic_tokens = tokens
                .iter()
                .filter_map(|token| {
                    let line = rope.try_byte_to_line(token.start).ok()? as u32;
                    let first = rope.try_line_to_char(line as usize).ok()? as u32;
                    let start = rope.try_byte_to_char(token.start).ok()? as u32 - first;
                    let delta_line = line - pre_line;
                    let delta_start = if delta_line == 0 {
                        start - pre_start
                    } else {
                        start
                    };
                    let ret = Some(SemanticToken {
                        delta_line,
                        delta_start,
                        length: token.length as u32,
                        token_type: token.token_type,
                        token_modifiers_bitset: 0,
                    });
                    pre_line = line;
                    pre_start = start;
                    ret
                })
                .collect::<Vec<_>>();
            Some(semantic_tokens)
        })();
        if let Some(semantic_token) = semantic_tokens {
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: semantic_token,
            })));
        }
        Ok(None)
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: DashMap::new(),
        }
    }
    pub async fn on_change(&self, params: TextDocumentItem) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
        let mut doc_info = DocumentInfo::new(Rope::from(params.text), None);
        yaml::parser::parse(&self, &mut doc_info);

        self.document_map
            .insert(params.uri.to_string(), doc_info.clone());

        // Log the diagnostics to the console.
        self.client
            .log_message(MessageType::INFO, format!("{:?}", &doc_info.diagnostics))
            .await;

        self.client
            .publish_diagnostics(
                params.uri,
                doc_info.clone().diagnostics,
                Some(params.version),
            )
            .await
    }
    /// Logs a message to the client in a separate async task.
    pub fn log(&self, message_type: MessageType, message: String) {
        let client = self.client.clone();
        tokio::spawn(async move {
            client.log_message(message_type, message).await;
        });
    }
    pub async fn log_async(&self, message_type: MessageType, message: String) {
        let client = self.client.clone();
        client.log_message(message_type, message).await;
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new).finish();

    service
        .inner()
        .client
        .log_message(MessageType::INFO, "Starting server...")
        .await;

    Server::new(stdin, stdout, socket).serve(service).await;
}
