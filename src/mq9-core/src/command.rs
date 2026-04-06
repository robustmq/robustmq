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
/// - `$mq9.AI.MAILBOX.{mail_id}.{high|normal|low}`
/// - `$mq9.AI.PUBLIC.LIST`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mq9Command {
    /// `$mq9.AI.MAILBOX.CREATE` — create a mailbox.
    MailboxCreate,
    /// `$mq9.AI.MAILBOX.{mail_id}.{priority}` — send/subscribe to a mailbox.
    Mailbox { mail_id: String, priority: Priority },
    /// `$mq9.AI.PUBLIC.LIST` — discover all public mailboxes.
    PublicList,
}

impl Mq9Command {
    pub fn is_mq9_subject(subject: &str) -> bool {
        Self::parse(subject).is_some()
    }

    /// Returns the full NATS subject string, e.g. `$mq9.AI.MAILBOX.m-001.normal`.
    pub fn to_subject(&self) -> String {
        match self {
            Mq9Command::MailboxCreate => format!("{}.MAILBOX.CREATE", PREFIX),
            Mq9Command::Mailbox { mail_id, priority } => {
                format!("{}.MAILBOX.{}.{}", PREFIX, mail_id, priority)
            }
            Mq9Command::PublicList => format!("{}.PUBLIC.LIST", PREFIX),
        }
    }

    /// Parse a full NATS subject string into an [`Mq9Subject`].
    pub fn parse(subject: &str) -> Option<Self> {
        let rest = subject.strip_prefix(PREFIX)?.strip_prefix('.')?;
        let parts: Vec<&str> = rest.splitn(4, '.').collect();
        match parts.as_slice() {
            ["MAILBOX", "CREATE"] => Some(Mq9Command::MailboxCreate),
            ["MAILBOX", mail_id, priority] => {
                let priority = Priority::from_str(priority)?;
                Some(Mq9Command::Mailbox {
                    mail_id: (*mail_id).to_string(),
                    priority,
                })
            }
            ["PUBLIC", "LIST"] => Some(Mq9Command::PublicList),
            _ => None,
        }
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
    fn test_roundtrip() {
        let cases = vec![
            Mq9Command::MailboxCreate,
            Mq9Command::Mailbox {
                mail_id: "m-uuid-001".to_string(),
                priority: Priority::High,
            },
            Mq9Command::Mailbox {
                mail_id: "m-uuid-001".to_string(),
                priority: Priority::Normal,
            },
            Mq9Command::Mailbox {
                mail_id: "m-uuid-001".to_string(),
                priority: Priority::Low,
            },
            Mq9Command::Mailbox {
                mail_id: "task.queue".to_string(),
                priority: Priority::Normal,
            },
            Mq9Command::PublicList,
        ];
        for s in &cases {
            assert_eq!(Mq9Command::parse(&s.to_subject()).as_ref(), Some(s));
        }
    }

    #[test]
    fn test_to_subject_strings() {
        assert_eq!(
            Mq9Command::MailboxCreate.to_subject(),
            "$mq9.AI.MAILBOX.CREATE"
        );
        assert_eq!(
            Mq9Command::Mailbox {
                mail_id: "m-001".to_string(),
                priority: Priority::High,
            }
            .to_subject(),
            "$mq9.AI.MAILBOX.m-001.high"
        );
        assert_eq!(Mq9Command::PublicList.to_subject(), "$mq9.AI.PUBLIC.LIST");
    }

    #[test]
    fn test_invalid() {
        assert_eq!(Mq9Command::parse("$mq9.AI.MAILBOX.m-001.urgent"), None);
        assert_eq!(Mq9Command::parse("$mq9.AI.INBOX.m-001.normal"), None);
        assert_eq!(Mq9Command::parse("$mq9.AI.BROADCAST.task.done"), None);
        assert_eq!(Mq9Command::parse("MAILBOX.CREATE"), None);
        assert_eq!(Mq9Command::parse("$mq9.AI.MAILBOX.CREATE.extra"), None);
    }

    #[test]
    fn test_is_mq9_subject() {
        assert!(Mq9Command::is_mq9_subject("$mq9.AI.MAILBOX.CREATE"));
        assert!(Mq9Command::is_mq9_subject("$mq9.AI.MAILBOX.m-001.normal"));
        assert!(Mq9Command::is_mq9_subject("$mq9.AI.PUBLIC.LIST"));
        assert!(!Mq9Command::is_mq9_subject("$mq9.AI.INBOX.m-001.normal"));
        assert!(!Mq9Command::is_mq9_subject("some.other.subject"));
    }
}
