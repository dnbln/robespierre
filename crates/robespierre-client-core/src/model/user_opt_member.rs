use robespierre_models::{
    id::{ServerId, UserId},
    servers::Member,
    users::User,
};

pub struct UserOptMember {
    pub user: User,
    pub member: Option<Member>,
}

impl UserOptMember {
    pub fn display_name(&self) -> &str {
        if let Some(Member {
            nickname: Some(nickname),
            ..
        }) = &self.member
        {
            return nickname;
        }

        &self.user.username.0
    }

    pub fn id(&self) -> UserId {
        self.user.id
    }

    pub fn server_id(&self) -> Option<ServerId> {
        self.member.as_ref().map(|it| it.id.server)
    }
}
