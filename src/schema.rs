// @generated automatically by Diesel CLI.

diesel::table! {
    dogs (id) {
        id -> Integer,
        name -> Text,
        image_path -> Text,
    }
}
