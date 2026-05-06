//! SSH Agent server — Unix socket / named pipe listener
//!
//! Implements the SSH agent protocol (RFC 4253) so that ssh, git, etc.
//! can use KeePassEx as their SSH agent.
//!
//! Socket path:
//!   macOS/Linux: $TMPDIR/keepassex-ssh-agent.sock
//!   Windows:     \\.\pipe\keepassex-ssh-agent

use crate::state::AppState;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info, warn};

#[cfg(unix)]
use tokio::net::UnixListener;

#[cfg(windows)]
use tokio::net::windows::named_pipe::ServerOptions;

/// SSH Agent message types (RFC 4253 / OpenSSH extension)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum AgentMsg {
    Failure = 5,
    Success = 6,
    RequestIdentities = 11,
    IdentitiesAnswer = 12,
    SignRequest = 13,
    SignResponse = 14,
    AddIdentity = 17,
    RemoveIdentity = 18,
    RemoveAllIdentities = 19,
}

/// Get the SSH agent socket path
pub fn socket_path() -> String {
    #[cfg(unix)]
    {
        let tmp = std::env::temp_dir();
        tmp.join("keepassex-ssh-agent.sock")
            .to_string_lossy()
            .to_string()
    }
    #[cfg(windows)]
    {
        r"\\.\pipe\keepassex-ssh-agent".to_string()
    }
    #[cfg(not(any(unix, windows)))]
    {
        "/tmp/keepassex-ssh-agent.sock".to_string()
    }
}

/// Start the SSH agent server
pub async fn start_server(state: Arc<AppState>) -> Result<String, String> {
    let path = socket_path();

    #[cfg(unix)]
    {
        // Remove stale socket
        let _ = tokio::fs::remove_file(&path).await;

        let listener = UnixListener::bind(&path)
            .map_err(|e| format!("Cannot bind SSH agent socket: {}", e))?;

        info!("SSH Agent listening on {}", path);

        let state_clone = state.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let state = state_clone.clone();
                        tokio::spawn(handle_connection(stream, state));
                    }
                    Err(e) => {
                        error!("SSH Agent accept error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(path)
    }

    #[cfg(not(unix))]
    {
        Err("SSH Agent is only supported on Unix/macOS/Linux in this version".into())
    }
}

/// Handle a single SSH agent connection
#[cfg(unix)]
async fn handle_connection(
    mut stream: tokio::net::UnixStream,
    state: Arc<AppState>,
) {
    loop {
        // Read 4-byte length prefix
        let mut len_buf = [0u8; 4];
        if stream.read_exact(&mut len_buf).await.is_err() {
            break;
        }
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        if msg_len == 0 || msg_len > 256 * 1024 {
            break;
        }

        // Read message body
        let mut msg = vec![0u8; msg_len];
        if stream.read_exact(&mut msg).await.is_err() {
            break;
        }

        // Process message
        let response = process_message(&msg, &state);

        // Write response with length prefix
        let len = (response.len() as u32).to_be_bytes();
        if stream.write_all(&len).await.is_err() {
            break;
        }
        if stream.write_all(&response).await.is_err() {
            break;
        }
    }
}

/// Process an SSH agent protocol message
fn process_message(msg: &[u8], state: &AppState) -> Vec<u8> {
    if msg.is_empty() {
        return vec![AgentMsg::Failure as u8];
    }

    let msg_type = msg[0];
    debug!("SSH Agent message type: {}", msg_type);

    match msg_type {
        // SSH2_AGENTC_REQUEST_IDENTITIES (11)
        11 => {
            let agent = state.ssh_agent.read().unwrap();
            let keys = agent.list_keys();

            let mut response = vec![AgentMsg::IdentitiesAnswer as u8];
            // Number of keys (4 bytes big-endian)
            let count = keys.len() as u32;
            response.extend_from_slice(&count.to_be_bytes());

            for key in keys {
                // Key blob (length-prefixed)
                let blob = key.public_key.as_bytes();
                let blob_len = blob.len() as u32;
                response.extend_from_slice(&blob_len.to_be_bytes());
                response.extend_from_slice(blob);

                // Comment (length-prefixed)
                let comment = key.comment.as_bytes();
                let comment_len = comment.len() as u32;
                response.extend_from_slice(&comment_len.to_be_bytes());
                response.extend_from_slice(comment);
            }

            response
        }

        // SSH2_AGENTC_REMOVE_ALL_IDENTITIES (19)
        19 => {
            state.ssh_agent.write().unwrap().remove_all();
            vec![AgentMsg::Success as u8]
        }

        // Unknown message
        _ => {
            warn!("SSH Agent: unknown message type {}", msg_type);
            vec![AgentMsg::Failure as u8]
        }
    }
}
