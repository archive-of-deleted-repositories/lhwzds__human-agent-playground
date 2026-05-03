use anyhow::{Context, Result, anyhow, bail};
use axum::body::Body;
use axum::extract::Request;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use hap_runtime::HumanAgentPlaygroundRuntime;
use human_agent_playground_backend::{build_api_router, runtime_config_from_env};
use reqwest::Method;
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::process::Command;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;

mod static_assets {
    include!(concat!(env!("OUT_DIR"), "/static_assets.rs"));
}

const DEFAULT_SERVER: &str = "http://127.0.0.1:8790";

#[derive(Debug)]
struct Options {
    server: String,
    pretty: bool,
    command: CommandSpec,
}

#[derive(Debug)]
enum CommandSpec {
    Serve {
        host: String,
        port: u16,
        open: bool,
    },
    Health,
    Games,
    SessionList,
    SessionCreate {
        game: String,
    },
    SessionShow {
        session_id: String,
    },
    LegalMoves {
        session_id: String,
        query: Option<Value>,
    },
    Play {
        session_id: String,
        move_input: Value,
    },
    Reset {
        session_id: String,
    },
    Watch {
        session_id: String,
    },
    Help,
}

#[tokio::main]
async fn main() -> Result<()> {
    let options = parse_args(std::env::args().skip(1).collect())?;
    match options.command {
        CommandSpec::Serve { host, port, open } => serve(&host, port, open).await,
        CommandSpec::Health => print_value(
            &request_json(&options.server, Method::GET, "/health", None).await?,
            options.pretty,
        ),
        CommandSpec::Games => print_value(
            &request_json(&options.server, Method::GET, "/api/games", None).await?,
            options.pretty,
        ),
        CommandSpec::SessionList => print_value(
            &request_json(&options.server, Method::GET, "/api/sessions", None).await?,
            options.pretty,
        ),
        CommandSpec::SessionCreate { game } => {
            let body = json!({
                "gameId": game,
                "actorKind": "agent",
                "channel": "http",
                "actorName": "hap",
            });
            print_value(
                &request_json(&options.server, Method::POST, "/api/sessions", Some(body)).await?,
                options.pretty,
            )
        }
        CommandSpec::SessionShow { session_id } => print_value(
            &request_json(
                &options.server,
                Method::GET,
                &format!("/api/sessions/{}", encode_segment(&session_id)),
                None,
            )
            .await?,
            options.pretty,
        ),
        CommandSpec::LegalMoves { session_id, query } => {
            let path = match query {
                Some(Value::Object(map)) if !map.is_empty() => {
                    let query = map
                        .iter()
                        .map(|(key, value)| {
                            let value = value
                                .as_str()
                                .map(str::to_string)
                                .unwrap_or_else(|| value.to_string());
                            format!("{}={}", encode_query(key), encode_query(&value))
                        })
                        .collect::<Vec<_>>()
                        .join("&");
                    format!(
                        "/api/sessions/{}/legal-moves?{}",
                        encode_segment(&session_id),
                        query
                    )
                }
                Some(_) => bail!("--query-json must be a JSON object"),
                None => format!("/api/sessions/{}/legal-moves", encode_segment(&session_id)),
            };
            print_value(
                &request_json(&options.server, Method::GET, &path, None).await?,
                options.pretty,
            )
        }
        CommandSpec::Play {
            session_id,
            move_input,
        } => {
            let mut body = match move_input {
                Value::Object(map) => map,
                _ => bail!("--move-json must be a JSON object"),
            };
            body.entry("actorKind".to_string())
                .or_insert_with(|| Value::String("agent".to_string()));
            body.entry("channel".to_string())
                .or_insert_with(|| Value::String("http".to_string()));
            body.entry("actorName".to_string())
                .or_insert_with(|| Value::String("hap".to_string()));

            print_value(
                &request_json(
                    &options.server,
                    Method::POST,
                    &format!("/api/sessions/{}/moves", encode_segment(&session_id)),
                    Some(Value::Object(body)),
                )
                .await?,
                options.pretty,
            )
        }
        CommandSpec::Reset { session_id } => {
            let body = json!({
                "actorKind": "agent",
                "channel": "http",
                "actorName": "hap",
            });
            print_value(
                &request_json(
                    &options.server,
                    Method::POST,
                    &format!("/api/sessions/{}/reset", encode_segment(&session_id)),
                    Some(body),
                )
                .await?,
                options.pretty,
            )
        }
        CommandSpec::Watch { session_id } => {
            watch(&options.server, &session_id, options.pretty).await
        }
        CommandSpec::Help => {
            print_usage();
            Ok(())
        }
    }
}

async fn serve(host: &str, port: u16, open: bool) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,human_agent_playground=info")),
        )
        .init();

    let runtime = HumanAgentPlaygroundRuntime::new(runtime_config_from_env()).await?;
    let cancellation = CancellationToken::new();
    let app = build_api_router(runtime, cancellation).fallback(static_asset_handler);
    let address: SocketAddr = format!("{host}:{port}")
        .parse()
        .with_context(|| format!("invalid listen address: {host}:{port}"))?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    let local_addr = listener.local_addr()?;
    let url = format!("http://{}:{}", host_for_url(host), local_addr.port());

    eprintln!("Human Agent Playground");
    eprintln!("  UI:  {url}/");
    eprintln!("  API: {url}/api");
    eprintln!("  MCP: {url}/mcp");

    if open {
        open_browser(&url);
    }

    axum::serve(listener, app).await?;
    Ok(())
}

async fn static_asset_handler(request: Request) -> Response {
    let uri: &Uri = request.uri();
    if is_reserved_api_path(uri.path()) {
        return StatusCode::NOT_FOUND.into_response();
    }

    let path = normalize_asset_path(uri.path());
    let asset = find_asset(path).or_else(|| find_asset("index.html"));

    match asset {
        Some(asset) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, asset.content_type)
            .body(Body::from(asset.bytes))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn request_json(
    server: &str,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<Value> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", server.trim_end_matches('/'), path);
    let mut request = client.request(method, &url);
    if let Some(body) = body {
        request = request.json(&body);
    }

    let response = request
        .send()
        .await
        .with_context(|| format!("failed to connect to {url}; run `hap serve` first"))?;
    let status = response.status();
    let text = response.text().await?;

    if !status.is_success() {
        bail!("request failed ({status}): {text}");
    }

    serde_json::from_str(&text).with_context(|| format!("invalid JSON response from {url}"))
}

async fn watch(server: &str, session_id: &str, pretty: bool) -> Result<()> {
    loop {
        let value = request_json(
            server,
            Method::GET,
            &format!("/api/sessions/{}", encode_segment(session_id)),
            None,
        )
        .await?;
        print_value(&value, pretty)?;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn parse_args(args: Vec<String>) -> Result<Options> {
    let mut args = args.into_iter().collect::<Vec<_>>();
    let mut server = std::env::var("HAP_SERVER_URL").unwrap_or_else(|_| DEFAULT_SERVER.to_string());
    let mut pretty = false;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--server" => {
                let value = args
                    .get(index + 1)
                    .cloned()
                    .ok_or_else(|| anyhow!("--server requires a value"))?;
                server = value;
                args.drain(index..=index + 1);
            }
            "--pretty" => {
                pretty = true;
                args.remove(index);
            }
            _ => {
                index += 1;
            }
        }
    }

    let command = parse_command(args)?;
    Ok(Options {
        server,
        pretty,
        command,
    })
}

fn parse_command(args: Vec<String>) -> Result<CommandSpec> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(CommandSpec::Help);
    };

    match command {
        "serve" => parse_serve(&args[1..]),
        "health" => Ok(CommandSpec::Health),
        "games" => Ok(CommandSpec::Games),
        "session" => parse_session(&args[1..]),
        "legal-moves" => parse_legal_moves(&args[1..]),
        "play" => parse_play(&args[1..]),
        "reset" => {
            let session_id = args
                .get(1)
                .cloned()
                .ok_or_else(|| anyhow!("reset requires <session-id>"))?;
            Ok(CommandSpec::Reset { session_id })
        }
        "watch" => {
            let session_id = args
                .get(1)
                .cloned()
                .ok_or_else(|| anyhow!("watch requires <session-id>"))?;
            Ok(CommandSpec::Watch { session_id })
        }
        "help" | "--help" | "-h" => Ok(CommandSpec::Help),
        value => bail!("unknown command: {value}"),
    }
}

fn parse_serve(args: &[String]) -> Result<CommandSpec> {
    let mut host = "127.0.0.1".to_string();
    let mut port = 8790;
    let mut open = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--host" => {
                host = args
                    .get(index + 1)
                    .cloned()
                    .ok_or_else(|| anyhow!("--host requires a value"))?;
                index += 2;
            }
            "--port" => {
                let raw = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("--port requires a value"))?;
                port = raw
                    .parse()
                    .with_context(|| format!("invalid port: {raw}"))?;
                index += 2;
            }
            "--open" => {
                open = true;
                index += 1;
            }
            value => bail!("unknown serve option: {value}"),
        }
    }

    Ok(CommandSpec::Serve { host, port, open })
}

fn parse_session(args: &[String]) -> Result<CommandSpec> {
    match args.first().map(String::as_str) {
        Some("list") => Ok(CommandSpec::SessionList),
        Some("create") => {
            let mut game = "chess".to_string();
            let mut index = 1;
            while index < args.len() {
                match args[index].as_str() {
                    "--game" => {
                        game = args
                            .get(index + 1)
                            .cloned()
                            .ok_or_else(|| anyhow!("--game requires a value"))?;
                        index += 2;
                    }
                    value => bail!("unknown session create option: {value}"),
                }
            }
            Ok(CommandSpec::SessionCreate { game })
        }
        Some("show") => {
            let session_id = args
                .get(1)
                .cloned()
                .ok_or_else(|| anyhow!("session show requires <session-id>"))?;
            Ok(CommandSpec::SessionShow { session_id })
        }
        Some(value) => bail!("unknown session command: {value}"),
        None => bail!("session requires a subcommand"),
    }
}

fn parse_legal_moves(args: &[String]) -> Result<CommandSpec> {
    let session_id = args
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("legal-moves requires <session-id>"))?;
    let mut query = None;
    let mut index = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--query-json" => {
                let raw = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("--query-json requires a value"))?;
                query = Some(serde_json::from_str(raw).context("invalid --query-json")?);
                index += 2;
            }
            value => bail!("unknown legal-moves option: {value}"),
        }
    }

    Ok(CommandSpec::LegalMoves { session_id, query })
}

fn parse_play(args: &[String]) -> Result<CommandSpec> {
    let session_id = args
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("play requires <session-id>"))?;
    let mut move_input = None;
    let mut index = 1;

    while index < args.len() {
        match args[index].as_str() {
            "--move-json" => {
                let raw = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("--move-json requires a value"))?;
                move_input = Some(serde_json::from_str(raw).context("invalid --move-json")?);
                index += 2;
            }
            value => bail!("unknown play option: {value}"),
        }
    }

    Ok(CommandSpec::Play {
        session_id,
        move_input: move_input.ok_or_else(|| anyhow!("play requires --move-json"))?,
    })
}

fn print_value(value: &Value, pretty: bool) -> Result<()> {
    if pretty {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("{}", serde_json::to_string(value)?);
    }
    Ok(())
}

fn print_usage() {
    eprintln!(
        "Usage:
  hap serve [--host 127.0.0.1] [--port 8790] [--open]
  hap health [--server URL]
  hap games [--server URL]
  hap session list [--server URL]
  hap session create [--game chess] [--server URL]
  hap session show <session-id> [--server URL]
  hap legal-moves <session-id> [--query-json '{{}}'] [--server URL]
  hap play <session-id> --move-json '{{...}}' [--server URL]
  hap reset <session-id> [--server URL]
  hap watch <session-id> [--server URL]

Global options:
  --server URL   Defaults to HAP_SERVER_URL or {DEFAULT_SERVER}
  --pretty       Pretty-print JSON output"
    );
}

fn normalize_asset_path(path: &str) -> &str {
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() {
        "index.html"
    } else {
        trimmed
    }
}

fn is_reserved_api_path(path: &str) -> bool {
    path == "/api" || path.starts_with("/api/") || path == "/mcp" || path.starts_with("/mcp/")
}

fn find_asset(path: &str) -> Option<&'static static_assets::StaticAsset> {
    static_assets::STATIC_ASSETS
        .iter()
        .find(|asset| asset.path == path)
}

fn host_for_url(host: &str) -> String {
    match host {
        "0.0.0.0" => "127.0.0.1".to_string(),
        "::" => "[::1]".to_string(),
        value if value.contains(':') && !value.starts_with('[') => format!("[{value}]"),
        value => value.to_string(),
    }
}

fn open_browser(url: &str) {
    let command = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "xdg-open"
    };

    let mut process = Command::new(command);
    if cfg!(target_os = "windows") {
        process.args(["/C", "start", url]);
    } else {
        process.arg(url);
    }
    if let Err(error) = process.spawn() {
        eprintln!("failed to open browser: {error}");
    }
}

fn encode_segment(value: &str) -> String {
    encode_query(value)
}

fn encode_query(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                output.push(byte as char);
            }
            _ => output.push_str(&format!("%{byte:02X}")),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_session_create_defaults() {
        let options = parse_args(vec!["session".into(), "create".into()]).unwrap();
        assert_eq!(options.server, DEFAULT_SERVER);
        assert!(!options.pretty);
        match options.command {
            CommandSpec::SessionCreate { game } => assert_eq!(game, "chess"),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_global_flags_anywhere() {
        let options = parse_args(vec![
            "games".into(),
            "--server".into(),
            "http://127.0.0.1:9000".into(),
            "--pretty".into(),
        ])
        .unwrap();
        assert_eq!(options.server, "http://127.0.0.1:9000");
        assert!(options.pretty);
        assert!(matches!(options.command, CommandSpec::Games));
    }

    #[test]
    fn falls_back_unknown_static_routes_to_index() {
        assert_eq!(normalize_asset_path("/"), "index.html");
        assert_eq!(normalize_asset_path("/sessions/abc"), "sessions/abc");
        assert!(is_reserved_api_path("/api/unknown"));
        assert!(is_reserved_api_path("/mcp/unknown"));
        assert!(!is_reserved_api_path("/sessions/abc"));
        assert!(find_asset("index.html").is_some());
    }
}
