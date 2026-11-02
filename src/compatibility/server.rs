//! Local A2A and MCP listeners for compatibility runs.

use std::sync::Arc;

use a2a_lab_dev_kit::{
    A2aClient, A2aLabError, A2aServer, AgentMessageFuture, AgentMessageHandler, AgentMessageReply,
    AgentMessageRequest, McpLab, McpServer, bind_local,
};

use super::fixtures::{standard_fixtures, unavailable_fixtures};
use super::lab::{FixtureLab, FixtureVariant, UnavailableLab, VariantLab};
use crate::ComplianceFixtures;

#[derive(Debug, Clone, Copy)]
pub enum FixtureMode {
    Variant(FixtureVariant),
    UnavailableCapabilities,
}

pub struct FixtureServer {
    pub a2a_url: String,
    pub mcp_url: String,
    pub fixtures: ComplianceFixtures,
    a2a_task: tokio::task::JoinHandle<()>,
    mcp_task: tokio::task::JoinHandle<()>,
}

impl FixtureServer {
    pub async fn start(mode: FixtureMode) -> Result<Self, A2aLabError> {
        let (fixtures, a2a_api, mcp_api) = match mode {
            FixtureMode::Variant(variant) => {
                let fixtures = standard_fixtures();
                fixtures
                    .validate()
                    .map_err(|error| A2aLabError::invalid("fixtures", error.to_string()))?;
                let core = FixtureLab::new().share();
                let a2a = Arc::clone(&core);
                let mcp = VariantLab::wrap(core, variant);
                (fixtures, a2a, mcp)
            }
            FixtureMode::UnavailableCapabilities => {
                let fixtures = unavailable_fixtures();
                fixtures
                    .validate()
                    .map_err(|error| A2aLabError::invalid("fixtures", error.to_string()))?;
                let api = UnavailableLab::share();
                let a2a = Arc::clone(&api);
                let mcp = api;
                (fixtures, a2a, mcp)
            }
        };

        let (a2a_listener, a2a_address) = bind_local().await?;
        let (mcp_listener, mcp_address) = bind_local().await?;
        let a2a_url = format!("http://{a2a_address}");
        let mcp_url = format!("http://{mcp_address}/mcp");

        let a2a_task = tokio::spawn(async move {
            let _ = A2aServer::new(&a2a_api)
                .with_message_handler(Arc::new(GroundedAgent))
                .listen(a2a_listener)
                .await;
        });
        let mcp_task = tokio::spawn(async move {
            let _ = McpServer::new(&mcp_api).serve_http(mcp_listener).await;
        });

        wait_until_ready(&a2a_url, &mcp_url).await?;

        Ok(Self {
            a2a_url,
            mcp_url,
            fixtures,
            a2a_task,
            mcp_task,
        })
    }

    pub async fn shutdown(self) {
        self.a2a_task.abort();
        self.mcp_task.abort();
        let _ = self.a2a_task.await;
        let _ = self.mcp_task.await;
    }
}

struct GroundedAgent;

impl AgentMessageHandler for GroundedAgent {
    fn handle(
        &self,
        request: AgentMessageRequest,
    ) -> AgentMessageFuture<'_, Result<AgentMessageReply, A2aLabError>> {
        Box::pin(async move {
            let task_id = request
                .text
                .split_once("exact task identifier `")
                .and_then(|(_, rest)| rest.split_once('`'))
                .map_or("tasks.discovered", |(task_id, _)| task_id);
            Ok(AgentMessageReply {
                text: format!("MCP list_tasks returned task `{task_id}`."),
            })
        })
    }
}

async fn wait_until_ready(a2a_url: &str, mcp_url: &str) -> Result<(), A2aLabError> {
    let a2a = A2aClient::new(a2a_url)?;
    let mut last = String::new();
    for _ in 0..100 {
        match a2a.agent_card().await {
            Ok(_) => {
                if McpLab::connect(mcp_url).await.is_ok() {
                    return Ok(());
                }
                "MCP did not accept a session".clone_into(&mut last);
            }
            Err(error) => last = error.to_string(),
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    Err(A2aLabError::unavailable(format!(
        "compatibility fixture readiness failed: {last}"
    )))
}
