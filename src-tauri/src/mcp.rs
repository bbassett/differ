use std::sync::Arc;
use tokio::sync::Mutex;

use rmcp::{
    handler::server::router::tool::ToolRouter,
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServerHandler,
};

use crate::state::CommentQueue;

#[derive(Clone)]
pub struct DifferMcpServer {
    queue: Arc<Mutex<CommentQueue>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl DifferMcpServer {
    pub fn new(queue: Arc<Mutex<CommentQueue>>) -> Self {
        Self {
            queue,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Get the next review comment from the queue. Returns the comment with file path, line range, code context, and the reviewer's feedback. Returns empty if no comments pending."
    )]
    async fn get_next_comment(&self) -> String {
        let mut queue = self.queue.lock().await;
        match queue.dequeue() {
            Some(comment) => {
                serde_json::to_string_pretty(&comment).unwrap_or_else(|_| "Serialization error".into())
            }
            None => "No comments pending.".into(),
        }
    }

    #[tool(description = "Get the number of pending review comments in the queue.")]
    async fn get_queue_status(&self) -> String {
        let queue = self.queue.lock().await;
        format!("{{\"pending\": {}}}", queue.len())
    }
}

#[tool_handler]
impl ServerHandler for DifferMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Differ review tool. Use get_next_comment to receive code review feedback. \
                 Each comment includes a file path, line range, code context, and the reviewer's instruction. \
                 Process comments one at a time."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

pub async fn start_mcp_server(queue: Arc<Mutex<CommentQueue>>, port: u16) -> Result<(), String> {
    use hyper_util::rt::TokioIo;
    use rmcp::transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    };

    let session_manager = Arc::new(LocalSessionManager::default());
    let config = StreamableHttpServerConfig::default();

    let queue_clone = queue.clone();
    let http_service = StreamableHttpService::new(
        move || Ok(DifferMcpServer::new(queue_clone.clone())),
        session_manager,
        config,
    );

    let addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind MCP server on port {}: {}", port, e))?;

    eprintln!("MCP server listening on http://{}", addr);

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .map_err(|e| format!("Accept error: {}", e))?;
        let io = TokioIo::new(stream);
        let svc = http_service.clone();

        tokio::spawn(async move {
            let svc =
                hyper::service::service_fn(
                    move |req: hyper::Request<hyper::body::Incoming>| {
                        let mut svc = svc.clone();
                        async move {
                            use tower::Service;
                            svc.call(req).await
                        }
                    },
                );
            if let Err(e) =
                hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
                    .serve_connection(io, svc)
                    .await
            {
                eprintln!("MCP connection error: {}", e);
            }
        });
    }
}
