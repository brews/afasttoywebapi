use crate::core;
use crate::schema::facilities;
use chrono::NaiveDate;
use diesel::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, Selectable, Insertable, Queryable, Serialize)]
#[diesel(table_name = facilities, check_for_backend(diesel::pg::Pg))]
pub struct Facility {
    pub uid: String,
    pub company: String,
    pub segment: String,
    pub technology: String,
    pub latitude: f32,
    pub longitude: f32,
    pub announcement_date: NaiveDate,
    pub estimated_investment: Option<i64>,
}

impl From<core::Facility> for Facility {
    fn from(item: core::Facility) -> Self {
        Facility {
            uid: item.uid,
            company: item.company,
            segment: item.segment,
            technology: item.technology,
            latitude: item.latitude,
            longitude: item.longitude,
            announcement_date: item.announcement_date,
            estimated_investment: item.estimated_investment,
        }
    }
}

impl From<Facility> for core::Facility {
    fn from(item: Facility) -> Self {
        core::Facility {
            uid: item.uid,
            company: item.company,
            segment: item.segment,
            technology: item.technology,
            latitude: item.latitude,
            longitude: item.longitude,
            announcement_date: item.announcement_date,
            estimated_investment: item.estimated_investment,
        }
    }
}
