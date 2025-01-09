// @generated automatically by Diesel CLI.

diesel::table! {
    facilities (uid) {
        uid -> Text,
        company -> Text,
        segment -> Text,
        technology -> Text,
        latitude -> Float4,
        longitude -> Float4,
        announcement_date -> Date,
        estimated_investment -> Nullable<Int8>,
    }
}
