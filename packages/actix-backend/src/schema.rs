// @generated automatically by Diesel CLI.
diesel::table! {
    map_download_events (id) {
        id -> Int8,
        map_id -> Text,
        client_ip -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        downloaded_at -> Timestamptz,
    }
}

diesel::table! {
    roles (name) {
        name -> Text,
        description -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        discord_id -> Text,
        username -> Text,
        avatar_url -> Nullable<Text>,
        email -> Nullable<Text>,
        role -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(users -> roles (role));

diesel::allow_tables_to_appear_in_same_query!(map_download_events, roles, users,);
