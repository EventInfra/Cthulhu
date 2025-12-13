use crate::web::WebState;
use crate::web::helpers::{DateTimeAgo, PortStatusExt};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use maud::{DOCTYPE, Markup, html};

pub async fn header(
    State(state): State<WebState>,
    Path(port_label): Path<String>,
) -> Result<Markup, Response> {
    let port = if let Some(v) = state.manager.get_port(&port_label).await {
        v
    } else {
        return Err((StatusCode::NOT_FOUND, "Port not found").into_response());
    };

    let ps = format!("{:?}", port.data.get_status());

    Ok(html! {
        table {
            tr {
                td {
                    b {
                        (port.data.label)
                    }
                }
            }
            tr {
                td {
                    "Current stage:"
                }
                td {
                    (port.data.get_current_stage().unwrap_or("UNKN"))
                }
                td {
                    "Current status:"
                }
                td style={"background-color: " (port.data.get_css_backgroundcolor())} {
                    (ps)
                }
                td {
                    "Start time:"
                }
                td {
                    (port.data.job_started.unwrap_or(Utc::now()).timeago())
                }
                td {
                    "Last update:"
                }
                td {
                    (port.data.get_last_updated().unwrap_or(Utc::now()).timeago())
                }
            }
            tr {
                td {
                    "Controls:"
                }
                td {
                    button onclick={ "abortJob('" (port.data.label) "')" } {
                        @if port.data.get_status().is_finished() {
                            "New Job"
                        } @else {
                            "Abort Job"
                        }
                    }
                }
            }
        }
    })
}
pub async fn footer(
    State(state): State<WebState>,
    Path(port_label): Path<String>,
) -> Result<Markup, Response> {
    let port = if let Some(v) = state.manager.get_port(&port_label).await {
        v
    } else {
        return Err((StatusCode::NOT_FOUND, "Port not found").into_response());
    };

    Ok(html! {
        table {
            tr {
                td {
                    h3 { "Device Information:"}
                    ul {
                        @for info in port.data.info_items.iter() {
                            li {
                                (info)
                            }
                        }
                    }
                }
                td {
                    h3 { "Stage history:" }
                    ul {
                        @for (t, stage) in port.data.state_history.iter().rev() {
                            li {
                                (stage) " (" (t.timeago()) ")"
                            }
                        }
                    }
                }
            }
        }
    })
}

pub async fn port(
    State(state): State<WebState>,
    Path(port_label): Path<String>,
) -> Result<Markup, Response> {
    let h = header(State(state.clone()), Path(port_label.clone())).await?;
    let f = footer(State(state.clone()), Path(port_label.clone())).await?;
    Ok(html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                link rel="stylesheet" href="/assets/css/xterm.css";
                link rel="stylesheet" href="/assets/css/port.css";
                script src="/assets/js/xterm.js" {}
                script src="/assets/js/addon-attach.js" {}
            }
            body {
                div id="header" {
                    (h)
                }
                div id="terminal" {}
                div id="devinfo" {
                    (f)
                }
                script src="/assets/js/port.js" {}
            }
        }
    })
}
