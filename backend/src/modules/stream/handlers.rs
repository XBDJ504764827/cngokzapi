use std::convert::Infallible;

use axum::{
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
};
use serde::Deserialize;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::{events::AppEvent, state::AppState};

#[derive(Deserialize)]
pub struct StreamQuery {
    /// 逗号分隔的订阅主题；为空表示订阅全部。
    pub topics: Option<String>,
}

/// GET /api/v1/stream —— SSE 长连接。
pub async fn stream(
    State(state): State<AppState>,
    Query(q): Query<StreamQuery>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let topics: Vec<String> = q
        .topics
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let rx = state.events.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |item| {
        let Ok(event) = item else {
            // 订阅者落后于广播缓冲区时跳过，不中断连接
            return None;
        };
        if !topics.is_empty() && !topics.iter().any(|t| t == &event.topic) {
            return None;
        }
        Some(Ok(format_event(&event)))
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

fn format_event(event: &AppEvent) -> Event {
    let data = serde_json::to_string(event).unwrap_or_default();
    Event::default().event("message").data(data)
}
