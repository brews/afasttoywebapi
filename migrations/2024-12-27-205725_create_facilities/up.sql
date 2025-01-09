CREATE TABLE facilities (
    uid TEXT PRIMARY KEY,
    company TEXT NOT NULL,
    segment TEXT NOT NULL,
    technology TEXT NOT NULL,
    latitude REAL NOT NULL,
    longitude REAL NOT NULL,
    announcement_date DATE NOT NULL,
    estimated_investment BIGINT
)