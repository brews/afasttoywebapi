use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_json_facility_with_investment() {
        let expected = Facility {
            uid: String::from("a_uid"),
            company: String::from("fancy company"),
            segment: String::from("some sector"),
            technology: String::from("fancy tech"),
            latitude: 80.5,
            longitude: -120.0,
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
            latitude: 80.5,
            longitude: -120.0,
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
