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
/// - `$mq9.AI.MAILBOX.MSG.{mail_address}.{critical|urgent|normal|low}` — publish
/// - `$mq9.AI.MAILBOX.MSG.{mail_address}`                               — subscribe (no priority)
/// - `$mq9.AI.MAILBOX.LIST.{mail_address}`
/// - `$mq9.AI.MAILBOX.DELETE.{mail_address}.{msg_id}`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mq9Command {
    /// `$mq9.AI.MAILBOX.CREATE`
    MailboxCreate,
    /// `$mq9.AI.MAILBOX.MSG.{mail_address}.{priority}` — publish a message.
    MailboxMsg {
        mail_address: String,
        priority: Priority,
    },
    /// `$mq9.AI.MAILBOX.MSG.{mail_address}` — subscribe to a mailbox (server pushes by priority).
    MailboxSub { mail_address: String },
    /// `$mq9.AI.MAILBOX.LIST.{mail_address}`
    MailboxList { mail_address: String },
    /// `$mq9.AI.MAILBOX.DELETE.{mail_address}.{msg_id}`
    MailboxDelete {
        mail_address: String,
        msg_id: String,
    },
}

impl Mq9Command {
    pub fn is_mq9_subject(subject: &str) -> bool {
        subject.starts_with(PREFIX)
    }

    pub fn to_subject(&self) -> String {
        match self {
            Mq9Command::MailboxCreate => format!("{}.MAILBOX.CREATE", PREFIX),
            Mq9Command::MailboxMsg {
                mail_address,
                priority,
            } => {
                format!("{}.MAILBOX.MSG.{}.{}", PREFIX, mail_address, priority)
            }
            Mq9Command::MailboxSub { mail_address } => {
                format!("{}.MAILBOX.MSG.{}", PREFIX, mail_address)
            }
            Mq9Command::MailboxList { mail_address } => {
                format!("{}.MAILBOX.LIST.{}", PREFIX, mail_address)
            }
            Mq9Command::MailboxDelete {
                mail_address,
                msg_id,
            } => {
                format!("{}.MAILBOX.DELETE.{}.{}", PREFIX, mail_address, msg_id)
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
                mail_address: tail.to_string(),
            }),
            "DELETE" if !tail.is_empty() => {
                // msg_id is always the last dot-separated token; mail_address is everything before.
                let (mail_address, msg_id) = tail.rsplit_once('.')?;
                Some(Mq9Command::MailboxDelete {
                    mail_address: mail_address.to_string(),
                    msg_id: msg_id.to_string(),
                })
            }
            _ => None,
        }
    }
}

/// Parse the tail after `$mq9.AI.MAILBOX.MSG.` into a [`Mq9Command`].
///
/// - Last segment is a known priority token → [`Mq9Command::MailboxMsg`] with that priority.
/// - No priority suffix (no dot, or last segment not a priority token) → [`Mq9Command::MailboxMsg`] with `Normal`.
/// - Subscribe uses a separate `SUB` packet and is matched via [`Mq9Command::MailboxSub`] directly.
fn parse_msg(tail: &str) -> Option<Mq9Command> {
    if tail.is_empty() {
        return None;
    }

    if let Some((prefix, last)) = tail.rsplit_once('.') {
        if let Some(p) = Priority::parse(last) {
            return Some(Mq9Command::MailboxMsg {
                mail_address: prefix.to_string(),
                priority: p,
            });
        }
        // last segment is not a priority token → entire tail is the mail_address
    }

    // No dot, or last segment not a priority → entire tail is mail_address, default Normal
    Some(Mq9Command::MailboxMsg {
        mail_address: tail.to_string(),
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
    fn test_parse() {
        // CREATE
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.CREATE"),
            Some(Mq9Command::MailboxCreate)
        );

        // MSG — no priority suffix → default Normal
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.MSG.m-001"),
            Some(Mq9Command::MailboxMsg {
                mail_address: "m-001".to_string(),
                priority: Priority::Normal
            })
        );
        // MSG — explicit priorities
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.MSG.m-001.urgent"),
            Some(Mq9Command::MailboxMsg {
                mail_address: "m-001".to_string(),
                priority: Priority::Urgent
            })
        );
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.MSG.m-001.critical"),
            Some(Mq9Command::MailboxMsg {
                mail_address: "m-001".to_string(),
                priority: Priority::Critical
            })
        );
        // MSG — mail_address with dots, last segment not a priority → entire tail is mail_address
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.MSG.task.queue"),
            Some(Mq9Command::MailboxMsg {
                mail_address: "task.queue".to_string(),
                priority: Priority::Normal
            })
        );

        // LIST
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.LIST.m-001"),
            Some(Mq9Command::MailboxList {
                mail_address: "m-001".to_string()
            })
        );

        // DELETE
        assert_eq!(
            Mq9Command::parse("$mq9.AI.MAILBOX.DELETE.m-001.msg-42"),
            Some(Mq9Command::MailboxDelete {
                mail_address: "m-001".to_string(),
                msg_id: "msg-42".to_string()
            })
        );

        // invalid
        assert_eq!(Mq9Command::parse("MAILBOX.CREATE"), None);
        assert_eq!(Mq9Command::parse("$mq9.AI.MAILBOX.CREATE.extra"), None);
        assert_eq!(Mq9Command::parse("$mq9.AI.MAILBOX.UNKNOWN.m-001"), None);
        assert_eq!(Mq9Command::parse("$mq9.AI.MAILBOX.MSG"), None);

        // is_mq9_subject
        assert!(Mq9Command::is_mq9_subject("$mq9.AI.MAILBOX.CREATE"));
        assert!(Mq9Command::is_mq9_subject(
            "$mq9.AI.MAILBOX.MSG.m-001.normal"
        ));
        assert!(!Mq9Command::is_mq9_subject("some.other.subject"));
    }
}
