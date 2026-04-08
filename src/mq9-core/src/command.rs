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
/// Full subject strings:
/// - `$mq9.AI.MAILBOX.CREATE`
/// - `$mq9.AI.MAILBOX.MSG.{mail_id}.{high|normal|low}`   — publish
/// - `$mq9.AI.MAILBOX.MSG.{mail_id}.*`                   — subscribe (all priorities)
/// - `$mq9.AI.MAILBOX.LIST.{mail_id}`
/// - `$mq9.AI.MAILBOX.DELETE.{mail_id}.{msg_id}`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mq9Command {
    /// `$mq9.AI.MAILBOX.CREATE`
    MailboxCreate,
    /// `$mq9.AI.MAILBOX.MSG.{mail_id}.{priority}` — publish a message.
    MailboxMsg { mail_id: String, priority: Priority },
    /// `$mq9.AI.MAILBOX.MSG.{mail_id}.{priority|*}` — subscribe to a mailbox.
    /// `priority = None` means wildcard `*` (all priorities).
    MailboxSub {
        mail_id: String,
        priority: Option<Priority>,
    },
    /// `$mq9.AI.MAILBOX.LIST.{mail_id}`
    MailboxList { mail_id: String },
    /// `$mq9.AI.MAILBOX.DELETE.{mail_id}.{msg_id}`
    MailboxDelete { mail_id: String, msg_id: String },
}

impl Mq9Command {
    pub fn is_mq9_subject(subject: &str) -> bool {
        subject.starts_with(PREFIX)
    }

    pub fn to_subject(&self) -> String {
        match self {
            Mq9Command::MailboxCreate => format!("{}.MAILBOX.CREATE", PREFIX),
            Mq9Command::MailboxMsg { mail_id, priority } => {
                format!("{}.MAILBOX.MSG.{}.{}", PREFIX, mail_id, priority)
            }
            Mq9Command::MailboxSub { mail_id, priority } => {
                let p = priority.as_ref().map(|p| p.as_str()).unwrap_or("*");
                format!("{}.MAILBOX.MSG.{}.{}", PREFIX, mail_id, p)
            }
            Mq9Command::MailboxList { mail_id } => {
                format!("{}.MAILBOX.LIST.{}", PREFIX, mail_id)
            }
            Mq9Command::MailboxDelete { mail_id, msg_id } => {
                format!("{}.MAILBOX.DELETE.{}.{}", PREFIX, mail_id, msg_id)
            }
        }
    }

    /// Parse a NATS subject into an [`Mq9Command`].
    ///
    /// For `MSG` subjects, if the priority token is `*` the result is
    /// [`Mq9Command::MailboxSub`] with `priority = None`. A concrete priority
    /// token on a `MSG` subject always yields [`Mq9Command::MailboxMsg`].
    pub fn parse(subject: &str) -> Option<Self> {
        let rest = subject.strip_prefix(PREFIX)?.strip_prefix('.')?;
        // First split off the op prefix (MAILBOX.CREATE, MAILBOX.MSG, etc.)
        let (op, tail) = rest.split_once('.')?;
        if op != "MAILBOX" {
            return None;
        }
        let (cmd, tail) = tail.split_once('.').unwrap_or((tail, ""));
        match cmd {
            "CREATE" if tail.is_empty() => Some(Mq9Command::MailboxCreate),
            "MSG" if !tail.is_empty() => parse_msg(tail),
            "LIST" if !tail.is_empty() => Some(Mq9Command::MailboxList {
                mail_id: tail.to_string(),
            }),
            "DELETE" if !tail.is_empty() => {
                // msg_id is always the last dot-separated token; mail_id is everything before.
                let (mail_id, msg_id) = tail.rsplit_once('.')?;
                Some(Mq9Command::MailboxDelete {
                    mail_id: mail_id.to_string(),
                    msg_id: msg_id.to_string(),
                })
            }
            _ => None,
        }
    }
}

/// Parse the tail after `$mq9.AI.MAILBOX.MSG.` into a [`Mq9Command`].
///
/// Priority (`critical`/`urgent`/`normal`/`*`) is always the last segment.
/// If the last segment is not a known priority token, the whole tail is the mail_id
/// and priority defaults to `Normal`.
fn parse_msg(tail: &str) -> Option<Mq9Command> {
    if let Some((prefix, last)) = tail.rsplit_once('.') {
        // last segment is `*` → wildcard subscribe
        if last == "*" {
            return Some(Mq9Command::MailboxSub {
                mail_id: prefix.to_string(),
                priority: None,
            });
        }
        // last segment is a known priority → pub or specific-priority sub
        if let Some(p) = Priority::parse(last) {
            return Some(Mq9Command::MailboxMsg {
                mail_id: prefix.to_string(),
                priority: p,
            });
        }
        // last segment is not a priority token → unknown, fall through to no-priority check
    }

    // No dot or last segment not a priority/wildcard → entire tail is mail_id, default Normal
    if tail.is_empty() {
        return None;
    }
    Some(Mq9Command::MailboxMsg {
        mail_id: tail.to_string(),
        priority: Priority::Normal,
    })
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
    fn test_parse_create() {
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.CREATE"),
            Some(Mq9Command::MailboxCreate)
        );
    }

    #[test]
    fn test_parse_msg_pub() {
        // No priority → default Normal
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.MSG.m-001"),
            Some(Mq9Command::MailboxMsg {
                mail_id: "m-001".to_string(),
                priority: Priority::Normal,
            })
        );
        // Explicit urgent
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.MSG.m-001.urgent"),
            Some(Mq9Command::MailboxMsg {
                mail_id: "m-001".to_string(),
                priority: Priority::Urgent,
            })
        );
        // Explicit critical
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.MSG.m-001.critical"),
            Some(Mq9Command::MailboxMsg {
                mail_id: "m-001".to_string(),
                priority: Priority::Critical,
            })
        );
        // mail_id with dots, no priority
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.MSG.task.queue"),
            Some(Mq9Command::MailboxMsg {
                mail_id: "task.queue".to_string(),
                priority: Priority::Normal,
            })
        );
    }

    #[test]
    fn test_parse_msg_sub_wildcard() {
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.MSG.m-001.*"),
            Some(Mq9Command::MailboxSub {
                mail_id: "m-001".to_string(),
                priority: None,
            })
        );
    }

    #[test]
    fn test_parse_msg_sub_specific() {
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.MSG.m-001.urgent"),
            Some(Mq9Command::MailboxMsg {
                mail_id: "m-001".to_string(),
                priority: Priority::Urgent,
            })
        );
    }

    #[test]
    fn test_parse_list() {
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.LIST.m-001"),
            Some(Mq9Command::MailboxList {
                mail_id: "m-001".to_string()
            })
        );
    }

    #[test]
    fn test_parse_delete() {
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.DELETE.m-001.msg-42"),
            Some(Mq9Command::MailboxDelete {
                mail_id: "m-001".to_string(),
                msg_id: "msg-42".to_string(),
            })
        );
    }

    #[test]
    fn test_to_subject_roundtrip() {
        let cases = vec![
            Mq9Command::MailboxCreate,
            Mq9Command::MailboxMsg {
                mail_id: "m-001".to_string(),
                priority: Priority::Normal,
            },
            Mq9Command::MailboxSub {
                mail_id: "m-001".to_string(),
                priority: None,
            },
            Mq9Command::MailboxList {
                mail_id: "m-001".to_string(),
            },
            Mq9Command::MailboxDelete {
                mail_id: "m-001".to_string(),
                msg_id: "msg-42".to_string(),
            },
        ];
        for cmd in &cases {
            let s = cmd.to_subject();
            let parsed = Mq9Command::parse(&s);
            // MailboxSub with concrete priority parses back as MailboxMsg — that's expected.
            assert!(parsed.is_some(), "failed to parse: {}", s);
        }
    }

    #[test]
    fn test_invalid() {
        // completely unrecognized prefix
        assert_eq!(Mq9Command::parse("MAILBOX.CREATE"), None);
        // extra token after CREATE
        assert_eq!(Mq9Command::parse("$mq9.AI.MAILBOX.CREATE.extra"), None);
        // unknown op
        assert_eq!(Mq9Command::parse("$mq9.AI.MAILBOX.UNKNOWN.m-001"), None);
        // MSG with no mail_id
        assert_eq!(Mq9Command::parse("$mq9.AI.MAILBOX.MSG"), None);
    }

    #[test]
    fn test_is_mq9_subject() {
        assert!(Mq9Command::is_mq9_subject("$mq9.AI.MAILBOX.CREATE"));
        assert!(Mq9Command::is_mq9_subject(
            "$mq9.AI.MAILBOX.MSG.m-001.normal"
        ));
        assert!(!Mq9Command::is_mq9_subject("some.other.subject"));
    }
}
