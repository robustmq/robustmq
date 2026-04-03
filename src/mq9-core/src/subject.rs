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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboxPriority {
    Urgent,
    Normal,
    Notify,
}

impl InboxPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            InboxPriority::Urgent => "urgent",
            InboxPriority::Normal => "normal",
            InboxPriority::Notify => "notify",
        }
    }
}

impl std::fmt::Display for InboxPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mq9Subject {
    MailboxCreate,
    MailboxQuery {
        mail_id: String,
    },
    Inbox {
        mail_id: String,
        priority: InboxPriority,
    },
    Broadcast {
        domain: String,
        event: String,
    },
}

impl Mq9Subject {
    pub fn is_mq9_subject(subject: &str) -> bool {
        Self::from_subject(subject).is_some()
    }

    pub fn to_subject(&self) -> String {
        match self {
            Mq9Subject::MailboxCreate => "MAILBOX.CREATE".to_string(),
            Mq9Subject::MailboxQuery { mail_id } => format!("MAILBOX.QUERY.{}", mail_id),
            Mq9Subject::Inbox { mail_id, priority } => format!("INBOX.{}.{}", mail_id, priority),
            Mq9Subject::Broadcast { domain, event } => format!("BROADCAST.{}.{}", domain, event),
        }
    }

    pub fn from_subject(subject: &str) -> Option<Self> {
        let parts: Vec<&str> = subject.splitn(4, '.').collect();
        match parts.as_slice() {
            ["MAILBOX", "CREATE"] => Some(Mq9Subject::MailboxCreate),
            ["MAILBOX", "QUERY", mail_id] => Some(Mq9Subject::MailboxQuery {
                mail_id: (*mail_id).to_string(),
            }),
            ["INBOX", mail_id, priority] => {
                let priority = match *priority {
                    "urgent" => InboxPriority::Urgent,
                    "normal" => InboxPriority::Normal,
                    "notify" => InboxPriority::Notify,
                    _ => return None,
                };
                Some(Mq9Subject::Inbox {
                    mail_id: (*mail_id).to_string(),
                    priority,
                })
            }
            [ns, domain, ..] if *ns == "BROADCAST" && parts.len() >= 3 => {
                Some(Mq9Subject::Broadcast {
                    domain: (*domain).to_string(),
                    event: parts[2..].join("."),
                })
            }
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
    fn test_subject_roundtrip() {
        let cases = vec![
            Mq9Subject::MailboxCreate,
            Mq9Subject::MailboxQuery {
                mail_id: "abc".to_string(),
            },
            Mq9Subject::Inbox {
                mail_id: "abc".to_string(),
                priority: InboxPriority::Urgent,
            },
            Mq9Subject::Broadcast {
                domain: "finance".to_string(),
                event: "order.created".to_string(),
            },
        ];
        for s in cases {
            assert_eq!(Mq9Subject::from_subject(&s.to_subject()), Some(s));
        }
    }

    #[test]
    fn test_invalid_subject() {
        assert_eq!(Mq9Subject::from_subject("UNKNOWN.foo"), None);
        assert_eq!(Mq9Subject::from_subject("INBOX.abc.bad"), None);
    }
}
