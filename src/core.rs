use chrono::NaiveDate;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Latitude(f32);

impl TryFrom<f32> for Latitude {
    type Error = LatitudeBoundsError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value < -90.0 || value > 90.0 {
            Err(LatitudeBoundsError)
        } else {
            Ok(Self(value))
        }
    }
}

impl From<Latitude> for f32 {
    fn from(value: Latitude) -> Self {
        f32::from(value.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct LatitudeBoundsError;

impl std::fmt::Display for LatitudeBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "outside of [-90.0, 90.0]")
    }
}

impl<'de> Deserialize<'de> for Latitude {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: f32 = Deserialize::deserialize(deserializer)?;
        Self::try_from(value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Longitude(f32);

impl TryFrom<f32> for Longitude {
    type Error = LongitudeBoundsError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value < -180.0 || value > 180.0 {
            Err(LongitudeBoundsError)
        } else {
            Ok(Self(value))
        }
    }
}

impl From<Longitude> for f32 {
    fn from(value: Longitude) -> Self {
        f32::from(value.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct LongitudeBoundsError;

impl std::fmt::Display for LongitudeBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "outside of [-180.0, 180.0]")
    }
}

impl<'de> Deserialize<'de> for Longitude {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: f32 = Deserialize::deserialize(deserializer)?;
        Self::try_from(value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Facility {
    pub uid: String,
    pub company: String,
    pub segment: String,
    pub technology: String,
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub announcement_date: NaiveDate,
    pub estimated_investment: Option<i64>,
}

impl Facility {
    pub fn new(
        uid: String,
        company: String,
        segment: String,
        technology: String,
        latitude: f32,
        longitude: f32,
        announcement_date: NaiveDate,
        estimated_investment: Option<i64>,
    ) -> Result<Self, FacilityError> {
        let lat = match Latitude::try_from(latitude) {
            Ok(r) => r,
            Err(LatitudeBoundsError) => return Err(FacilityError::LatitudeBounds),
        };

        let lon = match Longitude::try_from(longitude) {
            Ok(r) => r,
            Err(LongitudeBoundsError) => return Err(FacilityError::LongitudeBounds),
        };

        Ok(Facility {
            uid,
            company,
            segment,
            technology,
            latitude: lat,
            longitude: lon,
            announcement_date,
            estimated_investment,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum FacilityError {
    LatitudeBounds,
    LongitudeBounds,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn bad_latitude_error() {
        let facility_result = Facility::new(
            String::from("a_uid"),
            String::from("fancy company"),
            String::from("some sector"),
            String::from("fancy tech"),
            10000.0,
            150.0,
            NaiveDate::from_ymd_opt(2024, 12, 24).unwrap(),
            Some(123),
        );
        assert_eq!(facility_result, Err(FacilityError::LatitudeBounds));
    }

    #[test]
    fn bad_longitude_error() {
        let facility_result = Facility::new(
            String::from("a_uid"),
            String::from("fancy company"),
            String::from("some sector"),
            String::from("fancy tech"),
            80.5,
            10000.0,
            NaiveDate::from_ymd_opt(2024, 12, 24).unwrap(),
            Some(123),
        );
        assert_eq!(facility_result, Err(FacilityError::LongitudeBounds));
    }

    #[test]
    fn deserialize_json_facility_with_investment() {
        let expected = Facility {
            uid: String::from("a_uid"),
            company: String::from("fancy company"),
            segment: String::from("some sector"),
            technology: String::from("fancy tech"),
            latitude: Latitude::try_from(80.5).unwrap(),
            longitude: Longitude::try_from(-120.0).unwrap(),
            announcement_date: NaiveDate::from_ymd_opt(2024, 12, 24).unwrap(),
            estimated_investment: Some(123),
        };

        let json_facility = json!({
            "uid": "a_uid",
            "company": "fancy company",
            "segment": "some sector",
            "technology": "fancy tech",
            "latitude": 80.5,
            "longitude": -120.0,
            "announcement_date": "2024-12-24",
            "estimated_investment": 123
        });
        let actual: Facility = serde_json::from_value(json_facility).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialize_json_facility_without_investment() {
        // Testing when Facility.estimated_investment is None or "null". It can be tricky with JSON.
        let expected = Facility {
            uid: String::from("a_uid"),
            company: String::from("fancy company"),
            segment: String::from("some sector"),
            technology: String::from("fancy tech"),
            latitude: Latitude::try_from(80.5).unwrap(),
            longitude: Longitude::try_from(-120.0).unwrap(),
            announcement_date: NaiveDate::from_ymd_opt(2024, 12, 24).unwrap(),
            estimated_investment: None,
        };

        let json_facility = json!({
            "uid": "a_uid",
            "company": "fancy company",
            "segment": "some sector",
            "technology": "fancy tech",
            "latitude": 80.5,
            "longitude": -120.0,
            "announcement_date": "2024-12-24",
            "estimated_investment": null
        });
        let actual: Facility = serde_json::from_value(json_facility).unwrap();
        assert_eq!(actual, expected);
    }
}
