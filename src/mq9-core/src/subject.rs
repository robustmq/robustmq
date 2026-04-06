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

/// Subject namespace prefix: `$mq9.AI`
const PREFIX: &str = "$mq9.AI";

/// Priority levels for mailbox messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Priority {
    High,
    Normal,
    Low,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::High => "high",
            Priority::Normal => "normal",
            Priority::Low => "low",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "high" => Some(Priority::High),
            "normal" => Some(Priority::Normal),
            "low" => Some(Priority::Low),
            _ => None,
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// All recognized mq9 subjects.
///
/// Full subject strings:
/// - `$mq9.AI.MAILBOX.CREATE`
/// - `$mq9.AI.MAILBOX.{mail_id}.{high|normal|low}`
/// - `$mq9.AI.PUBLIC.LIST`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mq9Subject {
    /// `$mq9.AI.MAILBOX.CREATE` — create a mailbox.
    MailboxCreate,
    /// `$mq9.AI.MAILBOX.{mail_id}.{priority}` — send/subscribe to a mailbox.
    Mailbox { mail_id: String, priority: Priority },
    /// `$mq9.AI.PUBLIC.LIST` — discover all public mailboxes.
    PublicList,
}

impl Mq9Subject {
    pub fn is_mq9_subject(subject: &str) -> bool {
        Self::parse(subject).is_some()
    }

    /// Returns the full NATS subject string, e.g. `$mq9.AI.MAILBOX.m-001.normal`.
    pub fn to_subject(&self) -> String {
        match self {
            Mq9Subject::MailboxCreate => format!("{}.MAILBOX.CREATE", PREFIX),
            Mq9Subject::Mailbox { mail_id, priority } => {
                format!("{}.MAILBOX.{}.{}", PREFIX, mail_id, priority)
            }
            Mq9Subject::PublicList => format!("{}.PUBLIC.LIST", PREFIX),
        }
    }

    /// Parse a full NATS subject string into an [`Mq9Subject`].
    pub fn parse(subject: &str) -> Option<Self> {
        let rest = subject.strip_prefix(PREFIX)?.strip_prefix('.')?;
        let parts: Vec<&str> = rest.splitn(4, '.').collect();
        match parts.as_slice() {
            ["MAILBOX", "CREATE"] => Some(Mq9Subject::MailboxCreate),
            ["MAILBOX", mail_id, priority] => {
                let priority = Priority::from_str(priority)?;
                Some(Mq9Subject::Mailbox {
                    mail_id: (*mail_id).to_string(),
                    priority,
                })
            }
            ["PUBLIC", "LIST"] => Some(Mq9Subject::PublicList),
            _ => None,
        }
    }
}

impl std::fmt::Display for Mq9Subject {
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
            Mq9Subject::MailboxCreate,
            Mq9Subject::Mailbox {
                mail_id: "m-uuid-001".to_string(),
                priority: Priority::High,
            },
            Mq9Subject::Mailbox {
                mail_id: "m-uuid-001".to_string(),
                priority: Priority::Normal,
            },
            Mq9Subject::Mailbox {
                mail_id: "m-uuid-001".to_string(),
                priority: Priority::Low,
            },
            Mq9Subject::Mailbox {
                mail_id: "task.queue".to_string(),
                priority: Priority::Normal,
            },
            Mq9Subject::PublicList,
        ];
        for s in &cases {
            assert_eq!(Mq9Subject::parse(&s.to_subject()).as_ref(), Some(s));
        }
    }

    #[test]
    fn test_to_subject_strings() {
        assert_eq!(
            Mq9Subject::MailboxCreate.to_subject(),
            "$mq9.AI.MAILBOX.CREATE"
        );
        assert_eq!(
            Mq9Subject::Mailbox {
                mail_id: "m-001".to_string(),
                priority: Priority::High,
            }
            .to_subject(),
            "$mq9.AI.MAILBOX.m-001.high"
        );
        assert_eq!(Mq9Subject::PublicList.to_subject(), "$mq9.AI.PUBLIC.LIST");
    }

    #[test]
    fn test_invalid() {
        assert_eq!(Mq9Subject::parse("$mq9.AI.MAILBOX.m-001.urgent"), None);
        assert_eq!(Mq9Subject::parse("$mq9.AI.INBOX.m-001.normal"), None);
        assert_eq!(Mq9Subject::parse("$mq9.AI.BROADCAST.task.done"), None);
        assert_eq!(Mq9Subject::parse("MAILBOX.CREATE"), None);
        assert_eq!(Mq9Subject::parse("$mq9.AI.MAILBOX.CREATE.extra"), None);
    }

    #[test]
    fn test_is_mq9_subject() {
        assert!(Mq9Subject::is_mq9_subject("$mq9.AI.MAILBOX.CREATE"));
        assert!(Mq9Subject::is_mq9_subject("$mq9.AI.MAILBOX.m-001.normal"));
        assert!(Mq9Subject::is_mq9_subject("$mq9.AI.PUBLIC.LIST"));
        assert!(!Mq9Subject::is_mq9_subject("$mq9.AI.INBOX.m-001.normal"));
        assert!(!Mq9Subject::is_mq9_subject("some.other.subject"));
    }
}
