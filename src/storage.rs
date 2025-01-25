use crate::core;
use crate::models;
use crate::schema::facilities;
use chrono::NaiveDate;
use deadpool_diesel::postgres::{BuildError, Manager, Pool};
use deadpool_diesel::Runtime;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{de, Deserialize, Deserializer};
use std::fmt;
use std::str::FromStr;

/// Create async Postgres database connection pool.
pub fn create_database_connection_pool(
    database_url: String,
    max_size: usize,
) -> Result<Pool, BuildError> {
    let manager = Manager::new(database_url, Runtime::Tokio1);
    Pool::builder(manager).max_size(max_size).build()
}

/// Write Facility record to persistent storage, returning newly created facility if successful.
pub fn write_facility(
    conn: &mut PgConnection,
    facility: core::Facility,
) -> Result<core::Facility, diesel::result::Error> {
    // Return a copy of the facility we inserted. Doesn't play nicely with diesel, so using this hack as input/output should be the same for the moment.
    let returned_hack = facility.clone();
    let modeled_facility = models::Facility::from(facility);

    let output_result = diesel::insert_into(facilities::table)
        .values(modeled_facility)
        .execute(conn);

    let _ = match output_result {
        Ok(_) => (),
        Err(e) => return Err(e), // Error saving new facility.
    };
    // TODO: Fix leaking the abstraction by returning Err like this.

    // TODO: Handle the case of duplicate key: DatabaseError(UniqueViolation, "duplicate key value violates unique constraint \"facilities_pkey\"")
    // TODO: Query/return what we're inserting instead of 'returned_hack'.
    // TODO: Properly handle storage errors here.

    Ok(returned_hack)
}

/// Read a Facility record from persistent storage based on its UID.
pub fn read_facility(
    conn: &mut PgConnection,
    uid: String,
) -> Result<core::Facility, diesel::result::Error> {
    let db_output_results = facilities::table
        .filter(facilities::uid.eq(uid))
        .select(models::Facility::as_select())
        .first(conn);

    // TODO: Fix leaking the abstraction by returning Err like this.
    match db_output_results {
        Ok(r) => {
            let f: core::Facility = r
                .try_into()
                .expect("Got incompatible Facility from storage");
            // TODO: Actually pass and handle this error ^ properly
            Ok(f)
        }
        Err(e) => Err(e),
    }
}

/// List stored facilities.
pub fn list_facilities(
    conn: &mut PgConnection,
    filter: FacilitiesFilter,
) -> Result<Vec<core::Facility>, diesel::result::Error> {
    // A basic DB query we will build off of.
    let mut query = facilities::table.into_boxed::<diesel::pg::Pg>();

    // Optional filters may be added to query.
    if let Some(segment) = filter.segment {
        query = query.filter(facilities::segment.eq(segment));
    }
    if let Some(technology) = filter.technology {
        query = query.filter(facilities::technology.eq(technology));
    }
    if let Some(announced_before) = filter.announced_before {
        query = query.filter(facilities::announcement_date.lt(announced_before));
    }
    if let Some(announced_after) = filter.announced_after {
        query = query.filter(facilities::announcement_date.gt(announced_after));
    }

    // Add pagination
    query = query
        .offset(i64::from(filter.offset))
        .limit(i64::from(filter.limit));

    let db_output_results = query.select(models::Facility::as_select()).load(conn);

    match db_output_results {
        // Convert databases response into core::Facilities.
        Ok(r) => Ok(r
            .into_iter()
            .map(|x| {
                core::Facility::try_from(x).expect("Got incompatible Facility from storage")
                // TODO: Actually pass and handle this error ^ properly
            })
            .collect()),
        Err(e) => Err(e),
    }
}

/// Delete a Facility record from persistent storage based on its UID.
pub fn delete_facility(conn: &mut PgConnection, uid: String) -> Result<(), diesel::result::Error> {
    let query_result =
        diesel::delete(facilities::table.filter(facilities::uid.eq(uid))).execute(conn);

    let n_deleted = match query_result {
        Ok(r) => r,
        Err(e) => return Err(e),
    };

    if n_deleted == 0 {
        // Nothing was found. Nothing was deleted.
        return Err(diesel::result::Error::NotFound);
    }
    assert_eq!(n_deleted, 1);
    Ok(())
}

/// Filter list of facilities in storage.
#[derive(Debug, Deserialize)]
pub struct FacilitiesFilter {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub segment: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub technology: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub announced_before: Option<NaiveDate>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub announced_after: Option<NaiveDate>,
    #[serde(default = "default_offset")]
    pub offset: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    let limit: u32 = 100;
    limit
}

fn default_offset() -> u32 {
    let offset: u32 = 0;
    offset
}

/// Serde deserialization decorator to map empty Strings to None.
///
/// Needed so request parameters correctly deserialize as Option::None and not just "".
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}
