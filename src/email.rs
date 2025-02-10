use std::usize;

use log::{debug, error};
use mailparse::{DispositionType, ParsedMail};
use rumbok::{AllArgsConstructor, Getter};
use serde::{de::IntoDeserializer, Deserialize, Serialize};

#[derive(Serialize, AllArgsConstructor)]
pub struct Email {
    id: usize,
    content: String,
}

#[derive(Clone, Debug, Getter)]
pub struct EmailData {
    raw: String,
    body: String,
    subject: Option<String>,
    from: Option<String>,
    to: Option<String>,
    attachments: Vec<AttachmentData>,
}

#[derive(Serialize)]
pub struct EmailSummary {
    id: usize,
    subject: Option<String>,
    from: Option<String>,
    to: Option<String>,
    // raw データは詳細 API 用に保持
    raw: String,
    attachments: Vec<String>,
    body: String,
}

#[derive(Clone, Debug, Getter)]
pub struct AttachmentData {
    filename: Option<String>,
    content_type: String,
    data: String,
}

#[derive(Deserialize, Getter)]
pub struct SearchQuery {
    q : Option<String>,
}

impl EmailData {
    pub fn new(mail_content: String) -> Self {
        let parsed = mailparse::parse_mail(mail_content.as_bytes());

        let (subject, from, to, attachments, body) = if let Ok(parsed_mail) = parsed {
            Self::extract_headers(&parsed_mail)
        } else {
            (None, None, None, vec![], "".to_string())
        };

        Self {
            raw: mail_content,
            subject: subject,
            from: from,
            to: to,
            attachments: attachments,
            body: body,
        }
    }

    /// ヘッダー情報を抽出する補助関数
    fn extract_headers(
        parsed: &ParsedMail,
    ) -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Vec<AttachmentData>,
        String,
    ) {
        let mut subject = None;
        let mut from = None;
        let mut to = None;
        debug!("Mail header:{:?}", &parsed.headers);

        for header in &parsed.headers {
            let key = header.get_key_ref();
            let value = header.get_value();
            debug!("MailHeader: {}", &value);
            if key.eq_ignore_ascii_case("Subject") {
                subject = Some(value);
            } else if key.eq_ignore_ascii_case("From") {
                from = Some(value);
            } else if key.eq_ignore_ascii_case("To") {
                to = Some(value);
            }
        }

        (
            subject,
            from,
            to,
            AttachmentData::extract_attachement(parsed),
            Self::extract_body(parsed),
        )
    }

    fn extract_body(parsed: &ParsedMail) -> String {
        // multipart でなければ全体が本文とみなす
        if parsed.subparts.is_empty() {
            match parsed.get_body() {
                Ok(body) => {
                    return body;
                }
                Err(e) => {
                    error!("{:?}", e);
                    return "".to_string();
                }
            }
        } else {
            for subpart in &parsed.subparts {
                let mimetype = subpart.ctype.mimetype.to_lowercase();
                if mimetype == "text/plain"
                    && subpart.get_content_disposition().disposition == DispositionType::Inline
                {
                    if let Ok(body) = subpart.get_body() {
                        return body;
                    }
                }
            }
        }
        "".into()
    }

    pub fn convert_to_email_summary(&self, i: usize) -> EmailSummary {
        EmailSummary {
            id: i,
            subject: self.subject.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
            raw: self.raw.clone(),
            attachments: self
                .attachments
                .iter()
                .filter_map(|attachment| attachment.filename.clone())
                .collect(),
            body: self.body.clone(),
        }
    }
}

impl AttachmentData {
    pub fn extract_attachement(parsed: &ParsedMail) -> Vec<AttachmentData> {
        let mut attachments = vec![];
        for subpart in &parsed.subparts {
            let content_dispositon = subpart.get_content_disposition();
            debug!("{:?}", &content_dispositon);
            match content_dispositon.disposition {
                mailparse::DispositionType::Inline => {}
                mailparse::DispositionType::Attachment => {
                    let filename = content_dispositon.params.get("filename").cloned();

                    if let Ok(body) = subpart.get_body_raw() {
                        let data_b64 = base64::encode(&body);
                        attachments.push(Self {
                            filename: filename,
                            content_type: subpart.ctype.mimetype.clone(),
                            data: data_b64,
                        });
                    }
                }
                _ => {}
            }
        }

        attachments
    }
}
