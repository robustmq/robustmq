// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use metadata_struct::mq9::Priority;

/// Subject namespace prefix: `$mq9.AI`
const PREFIX: &str = "$mq9.AI";

/// All recognized mq9 subjects.
///
/// Protocol layout:
///
/// Mailbox management:
///   $mq9.AI.MAILBOX.CREATE
///
/// Message communication:
///   $mq9.AI.MSG.SEND.{mail_address}
///   $mq9.AI.MSG.SEND.{mail_address}.urgent
///   $mq9.AI.MSG.SEND.{mail_address}.critical
///   $mq9.AI.MSG.SUB.{mail_address}
///   $mq9.AI.MSG.ACK.{mail_address}.{msg_id}
///   $mq9.AI.MSG.QUERY.{mail_address}
///   $mq9.AI.MSG.DELETE.{mail_address}.{msg_id}
///
/// Agent management:
///   $mq9.AI.AGENT.REGISTER
///   $mq9.AI.AGENT.UNREGISTER
///   $mq9.AI.AGENT.REPORT
///   $mq9.AI.AGENT.DISCOVER
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mq9Command {
    // ── Mailbox management ────────────────────────────────────────────────────
    /// `$mq9.AI.MAILBOX.CREATE`
    MailboxCreate,

    // ── Message communication ─────────────────────────────────────────────────
    /// `$mq9.AI.MSG.SEND.{mail_address}[.urgent|.critical]`
    MsgSend {
        mail_address: String,
        priority: Priority,
    },
    /// `$mq9.AI.MSG.SUB.{mail_address}`
    MsgSub { mail_address: String },
    /// `$mq9.AI.MSG.ACK.{mail_address}.{msg_id}`
    MsgAck {
        mail_address: String,
        msg_id: String,
    },
    /// `$mq9.AI.MSG.QUERY.{mail_address}`
    MsgQuery { mail_address: String },
    /// `$mq9.AI.MSG.DELETE.{mail_address}.{msg_id}`
    MsgDelete {
        mail_address: String,
        msg_id: String,
    },

    // ── Agent management ──────────────────────────────────────────────────────
    /// `$mq9.AI.AGENT.REGISTER`
    AgentRegister,
    /// `$mq9.AI.AGENT.UNREGISTER`
    AgentUnregister,
    /// `$mq9.AI.AGENT.REPORT`
    AgentReport,
    /// `$mq9.AI.AGENT.DISCOVER`
    AgentDiscover,
}

impl Mq9Command {
    pub fn is_mq9_subject(subject: &str) -> bool {
        subject.starts_with(PREFIX)
    }

    /// Returns the fixed subject prefix for `MsgSub`, i.e. `"$mq9.AI.MSG.SUB."`.
    /// Use this to strip the prefix and extract the mail_address from a SUB subject.
    pub fn msg_sub_prefix() -> &'static str {
        concat!("$mq9.AI", ".MSG.SUB.")
    }

    pub fn to_subject(&self) -> String {
        match self {
            Mq9Command::MailboxCreate => format!("{}.MAILBOX.CREATE", PREFIX),

            Mq9Command::MsgSend {
                mail_address,
                priority,
            } => match priority {
                Priority::Normal => format!("{}.MSG.SEND.{}", PREFIX, mail_address),
                p => format!("{}.MSG.SEND.{}.{}", PREFIX, mail_address, p),
            },
            Mq9Command::MsgSub { mail_address } => {
                format!("{}.MSG.SUB.{}", PREFIX, mail_address)
            }
            Mq9Command::MsgAck {
                mail_address,
                msg_id,
            } => format!("{}.MSG.ACK.{}.{}", PREFIX, mail_address, msg_id),
            Mq9Command::MsgQuery { mail_address } => {
                format!("{}.MSG.QUERY.{}", PREFIX, mail_address)
            }
            Mq9Command::MsgDelete {
                mail_address,
                msg_id,
            } => format!("{}.MSG.DELETE.{}.{}", PREFIX, mail_address, msg_id),

            Mq9Command::AgentRegister => format!("{}.AGENT.REGISTER", PREFIX),
            Mq9Command::AgentUnregister => format!("{}.AGENT.UNREGISTER", PREFIX),
            Mq9Command::AgentReport => format!("{}.AGENT.REPORT", PREFIX),
            Mq9Command::AgentDiscover => format!("{}.AGENT.DISCOVER", PREFIX),
        }
    }

    /// Parse a NATS subject into an [`Mq9Command`].
    pub fn parse(subject: &str) -> Option<Self> {
        let rest = subject.strip_prefix(PREFIX)?.strip_prefix('.')?;
        let (namespace, tail) = rest.split_once('.')?;

        match namespace {
            "MAILBOX" => parse_mailbox(tail),
            "MSG" => parse_msg(tail),
            "AGENT" => parse_agent(tail),
            _ => None,
        }
    }
}

fn parse_mailbox(tail: &str) -> Option<Mq9Command> {
    match tail {
        "CREATE" => Some(Mq9Command::MailboxCreate),
        _ => None,
    }
}

fn parse_msg(tail: &str) -> Option<Mq9Command> {
    let (cmd, rest) = tail.split_once('.')?;
    if rest.is_empty() {
        return None;
    }
    match cmd {
        "SEND" => Some(parse_msg_send(rest)),
        "SUB" => Some(Mq9Command::MsgSub {
            mail_address: rest.to_string(),
        }),
        "ACK" => {
            let (mail_address, msg_id) = rest.rsplit_once('.')?;
            Some(Mq9Command::MsgAck {
                mail_address: mail_address.to_string(),
                msg_id: msg_id.to_string(),
            })
        }
        "QUERY" => Some(Mq9Command::MsgQuery {
            mail_address: rest.to_string(),
        }),
        "DELETE" => {
            let (mail_address, msg_id) = rest.rsplit_once('.')?;
            Some(Mq9Command::MsgDelete {
                mail_address: mail_address.to_string(),
                msg_id: msg_id.to_string(),
            })
        }
        _ => None,
    }
}

/// Parse the tail after `$mq9.AI.MSG.SEND.` into a [`Mq9Command::MsgSend`].
///
/// Priority is carried as the last segment if it matches a known token;
/// otherwise the entire tail is the mail_address with Normal priority.
fn parse_msg_send(tail: &str) -> Mq9Command {
    if let Some((prefix, last)) = tail.rsplit_once('.') {
        if let Some(p) = Priority::parse(last) {
            return Mq9Command::MsgSend {
                mail_address: prefix.to_string(),
                priority: p,
            };
        }
    }
    Mq9Command::MsgSend {
        mail_address: tail.to_string(),
        priority: Priority::Normal,
    }
}

fn parse_agent(tail: &str) -> Option<Mq9Command> {
    match tail {
        "REGISTER" => Some(Mq9Command::AgentRegister),
        "UNREGISTER" => Some(Mq9Command::AgentUnregister),
        "REPORT" => Some(Mq9Command::AgentReport),
        "DISCOVER" => Some(Mq9Command::AgentDiscover),
        _ => None,
    }
}

impl std::fmt::Display for Mq9Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_subject())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mailbox_create() {
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.CREATE"),
            Some(Mq9Command::MailboxCreate)
        );
    }

    #[test]
    fn test_msg_send() {
        // no priority suffix → Normal
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MSG.SEND.task.001.callback"),
            Some(Mq9Command::MsgSend {
                mail_address: "task.001.callback".to_string(),
                priority: Priority::Normal,
            })
        );
        // urgent
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MSG.SEND.agent.inbox.urgent"),
            Some(Mq9Command::MsgSend {
                mail_address: "agent.inbox".to_string(),
                priority: Priority::Urgent,
            })
        );
        // critical
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MSG.SEND.agent.inbox.critical"),
            Some(Mq9Command::MsgSend {
                mail_address: "agent.inbox".to_string(),
                priority: Priority::Critical,
            })
        );
    }

    #[test]
    fn test_msg_sub() {
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MSG.SUB.task.001.callback"),
            Some(Mq9Command::MsgSub {
                mail_address: "task.001.callback".to_string(),
            })
        );
    }

    #[test]
    fn test_msg_ack() {
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MSG.ACK.task.001.callback.42"),
            Some(Mq9Command::MsgAck {
                mail_address: "task.001.callback".to_string(),
                msg_id: "42".to_string(),
            })
        );
    }

    #[test]
    fn test_msg_query() {
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MSG.QUERY.task.001.callback"),
            Some(Mq9Command::MsgQuery {
                mail_address: "task.001.callback".to_string(),
            })
        );
    }

    #[test]
    fn test_msg_delete() {
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MSG.DELETE.task.001.callback.7"),
            Some(Mq9Command::MsgDelete {
                mail_address: "task.001.callback".to_string(),
                msg_id: "7".to_string(),
            })
        );
    }

    #[test]
    fn test_agent_commands() {
        assert_eq!(
            Mq9Command::parse("$mq9.AI.AGENT.REGISTER"),
            Some(Mq9Command::AgentRegister)
        );
        assert_eq!(
            Mq9Command::parse("$mq9.AI.AGENT.UNREGISTER"),
            Some(Mq9Command::AgentUnregister)
        );
        assert_eq!(
            Mq9Command::parse("$mq9.AI.AGENT.REPORT"),
            Some(Mq9Command::AgentReport)
        );
        assert_eq!(
            Mq9Command::parse("$mq9.AI.AGENT.DISCOVER"),
            Some(Mq9Command::AgentDiscover)
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(Mq9Command::parse("MAILBOX.CREATE"), None);
        assert_eq!(Mq9Command::parse("$mq9.AI.UNKNOWN.FOO"), None);
        assert_eq!(Mq9Command::parse("$mq9.AI.MSG.SEND"), None);
        assert_eq!(Mq9Command::parse("$mq9.AI.MSG.UNKNOWN.addr"), None);
        assert_eq!(Mq9Command::parse("$mq9.AI.AGENT.UNKNOWN"), None);
    }

    #[test]
    fn test_is_mq9_subject() {
        assert!(Mq9Command::is_mq9_subject("$mq9.AI.MAILBOX.CREATE"));
        assert!(Mq9Command::is_mq9_subject("$mq9.AI.MSG.SEND.foo"));
        assert!(Mq9Command::is_mq9_subject("$mq9.AI.AGENT.REGISTER"));
        assert!(!Mq9Command::is_mq9_subject("some.other.subject"));
    }

    #[test]
    fn test_to_subject_roundtrip() {
        let cases = vec![
            Mq9Command::MailboxCreate,
            Mq9Command::MsgSend {
                mail_address: "agent.inbox".to_string(),
                priority: Priority::Normal,
            },
            Mq9Command::MsgSend {
                mail_address: "agent.inbox".to_string(),
                priority: Priority::Urgent,
            },
            Mq9Command::MsgSub {
                mail_address: "task.001".to_string(),
            },
            Mq9Command::MsgQuery {
                mail_address: "task.001".to_string(),
            },
            Mq9Command::AgentRegister,
            Mq9Command::AgentDiscover,
        ];
        for cmd in cases {
            let subject = cmd.to_subject();
            let parsed = Mq9Command::parse(&subject);
            assert_eq!(
                parsed,
                Some(cmd.clone()),
                "roundtrip failed for {}",
                subject
            );
        }
    }
}
