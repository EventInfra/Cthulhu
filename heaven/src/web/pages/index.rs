use axum::extract::State;
use maud::{html, Markup, DOCTYPE};
use crate::web::helpers::*;
use crate::web::WebState;

pub async fn index(s: State<WebState>) -> Markup {
    let ps = port_status(s).await;
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                link rel="stylesheet" href="/assets/css/index.css";
                script src="/assets/js/index.js" {}
            }
            body id="portstatus" {
                (ps)
            }
        }
    }
}
pub async fn port_status(State(state): State<WebState>) -> Markup {
    let mut ports = state.manager.get_ports().await;
    ports.sort_by(|a, b| a.data.label.cmp(&b.data.label));

    let column_height = 8usize.min(ports.len());
    let total_columns = ((ports.len() as f64) / (column_height as f64)).ceil() as usize;

    html! {
        table class="outer" {
            @for i in 0..column_height {
                tr {
                    @for j in 0..total_columns {
                        td {
                            @let port_n = (j * column_height) + i;
                            @if port_n < ports.len() {
                                @let port = &ports[port_n];
                                table class="inner" style={"background-color: " (port.data.get_css_backgroundcolor())} {
                                    tr {
                                        td {
                                            (port.data.get_last_updated().map(|v| v.timeago()).unwrap_or("UNKN".to_string()))
                                        }
                                        td {
                                            b {
                                                a href={ "/port/" (port.data.label) "/" } target="_blank" {
                                                    (port.data.label)
                                                }
                                            }
                                        }
                                        td {
                                            (port.data.get_status())
                                        }
                                    }
                                    tr {
                                        td {
                                            (get_dev_manuf(&port.data))
                                        }
                                        td {
                                            (get_dev_model(&port.data))
                                        }
                                        td {
                                            (get_dev_sn(&port.data))
                                        }
                                    }
                                    tr {
                                        td colspan="3" {
                                            (port.data.get_current_stage().unwrap_or("UNKN"))
                                        }
                                    }
                                    tr {
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
                            }
                        }
                    }
                }
            }
        }
    }
}