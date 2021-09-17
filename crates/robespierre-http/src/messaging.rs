use robespierre_models::{
    autumn::AttachmentId,
    channels::{Message, MessageFilter, ReplyData},
    id::{ChannelId, MessageId},
    servers::Member,
    users::User,
};

use super::impl_prelude::*;

impl Http {
    /// Sends a message
    pub async fn send_message(
        &self,
        channel_id: ChannelId,
        content: impl AsRef<str>,
        nonce: impl AsRef<str>,
        attachments: Vec<AttachmentId>,
        replies: Vec<ReplyData>,
    ) -> Result<Message> {
        Ok(self
            .client
            .post(ep!(self, "/channels/{}/messages" channel_id))
            .json(&SendMessageRequest {
                content: content.as_ref(),
                nonce: nonce.as_ref(),
                attachments,
                replies,
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Fetches messages
    pub async fn fetch_messages(
        &self,
        channel: ChannelId,
        filter: MessageFilter,
    ) -> Result<FetchMessagesResult> {
        let v = self
            .client
            .get(ep!(self, "/channels/{}/messages" channel))
            .json(&filter)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        if matches!(&filter.include_users, Some(true)) {
            Ok(serde_json::from_value(v)?)
        } else {
            Ok(FetchMessagesResult {
                messages: serde_json::from_value(v)?,
                users: vec![],
                members: vec![],
            })
        }
    }

    /// Fetches a single message
    pub async fn fetch_message(&self, channel: ChannelId, message: MessageId) -> Result<Message> {
        Ok(self
            .client
            .get(ep!(self, "/channels/{}/messages/{}" channel, message))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Edits a message to contain content `content`
    pub async fn edit_message(
        &self,
        channel: ChannelId,
        message: MessageId,
        content: &str,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct MessagePatch<'a> {
            content: &'a str,
        }
        self.client
            .patch(ep!(self, "/channels/{}/messages/{}" channel, message))
            .json(&MessagePatch { content })
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Deletes a message
    pub async fn delete_message(&self, channel: ChannelId, message: MessageId) -> Result {
        self.client
            .delete(ep!(self, "/channels/{}/messages/{}" channel, message))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn poll_message_changes(
        &self,
        channel: ChannelId,
        ids: &[MessageId],
    ) -> Result<PollMessageChangesResponse> {
        #[derive(serde::Serialize)]
        struct PollMessageChanges<'a> {
            ids: &'a [MessageId],
        }

        Ok(self
            .client
            .post(ep!(self, "/channels/{}/messages/stale" channel))
            .json(&PollMessageChanges { ids })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // TODO: search for messages

    pub async fn acknowledge_message(&self, channel: ChannelId, message: MessageId) -> Result {
        self.client_user_session_auth_type()
            .put(ep!(self, "/channels/{}/ack/{}" channel, message))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
pub struct SendMessageRequest<'a> {
    pub content: &'a str,
    pub nonce: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<AttachmentId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub replies: Vec<ReplyData>,
}

#[derive(serde::Deserialize)]
pub struct PollMessageChangesResponse {
    pub changed: Vec<Message>,
    pub deleted: Vec<MessageId>,
}

/// Result when fetching multiple messages
#[derive(serde::Deserialize)]
pub struct FetchMessagesResult {
    pub messages: Vec<Message>,
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub members: Vec<Member>,
}
