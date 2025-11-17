use crate::{
    domain::models::{
        MessageHistoryEntry, MessageStatus, MessengerChat, MessengerToken, MessengerTokenStatus,
    },
    presentation::{
        http::responses::{
            MessageHistoryDto, MessengerChatDto, MessengerTokenDto, MessengerTokenStatusDto,
        },
        models::{ChatTypeKind, MessageStatusDto},
    },
};

pub fn map_token(token: &MessengerToken) -> MessengerTokenDto {
    MessengerTokenDto {
        id: token.id,
        messenger: token.messenger.into(),
        status: match token.status {
            MessengerTokenStatus::Active => MessengerTokenStatusDto::Active,
            MessengerTokenStatus::Inactive => MessengerTokenStatusDto::Inactive,
        },
        updated_at: token.updated_at.to_rfc3339(),
    }
}

pub fn map_history(entry: &MessageHistoryEntry) -> MessageHistoryDto {
    MessageHistoryDto {
        id: entry.id,
        messenger: entry.messenger.into(),
        recipient: entry.recipient.clone(),
        status: MessageStatusDto::from(&entry.status),
        attempts: entry.attempts,
        body: entry.content.body.clone(),
        last_error: extract_error(&entry.status),
        requested_by: entry.requested_by.clone().into(),
        created_at: entry.created_at.to_rfc3339(),
        updated_at: entry.updated_at.to_rfc3339(),
    }
}

fn extract_error(status: &MessageStatus) -> Option<String> {
    match status {
        MessageStatus::Retrying { reason, .. } => Some(reason.clone()),
        MessageStatus::Failed { reason, .. } => Some(reason.clone()),
        _ => None,
    }
}

pub fn map_chat(chat: &MessengerChat) -> MessengerChatDto {
    MessengerChatDto {
        messenger: chat.messenger.into(),
        chat_id: chat.chat_id.clone(),
        title: chat.title.clone(),
        chat_type: ChatTypeKind::from(chat.chat_type.clone()),
        can_send_messages: chat.can_send_messages,
    }
}
