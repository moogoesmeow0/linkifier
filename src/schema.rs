// @generated automatically by Diesel CLI.

diesel::table! {
    links (link) {
        #[max_length = 512]
        link -> Varchar,
        created_at -> Timestamp,
        #[max_length = 512]
        redirect -> Varchar,
    }
}
