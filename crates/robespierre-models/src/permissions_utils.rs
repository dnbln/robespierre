use crate::{
    channel::{Channel, ChannelPermissions},
    id::RoleId,
    server::{Member, Role, Server, ServerPermissions},
};

fn perms_or(server: &Server, roles: &[RoleId]) -> (ServerPermissions, ChannelPermissions) {
    if let Some(ref roles_obj) = server.roles {
        roles_obj.iter().filter(|it| roles.contains(it.0)).fold(
            server.default_permissions,
            |p, (_, Role { permissions, .. })| (p.0 | permissions.0, p.1 | permissions.1),
        )
    } else {
        server.default_permissions
    }
}

fn perms_or_in_channel(
    server: &Server,
    roles: &[RoleId],
    channel: &Channel,
) -> (ServerPermissions, ChannelPermissions) {
    let base = perms_or(server, roles);

    match channel {
        Channel::TextChannel {
            default_permissions,
            role_permissions,
            ..
        }
        | Channel::VoiceChannel {
            default_permissions,
            role_permissions,
            ..
        } => {
            let new = (
                base.0,
                base.1 | default_permissions.unwrap_or(ChannelPermissions::empty()),
            );

            let new = role_permissions
                .iter()
                .filter(|it| roles.contains(it.0))
                .fold(new, |p, (_, new)| (p.0, p.1 | *new));

            new
        }

        _ => unreachable!(),
    }
}

pub fn member_has_permissions(
    member: &Member,
    server_permissions: ServerPermissions,
    server: &Server,
) -> bool {
    if server.owner == member.id.user {
        // owner has all perms
        return true
    }

    perms_or(server, &member.roles).0.contains(server_permissions)
}

pub fn member_has_permissions_in_channel(
    member: &Member,
    server_permissions: ServerPermissions,
    server: &Server,
    channel_permissions: ChannelPermissions,
    channel: &Channel,
) -> bool {
    if server.owner == member.id.user {
        // owner has all perms
        return true
    }

    let (sp, cp) = perms_or_in_channel(server, &member.roles, channel);

    sp.contains(server_permissions) && cp.contains(channel_permissions)
}
