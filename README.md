# afasttoywebapi

## Some manual server tests
Run from a terminal shell:

```shell
SERVER_URL="http://localhost:8080"

# Post a facility
curl -i --location --request POST "${SERVER_URL}/facilities" \
  --header 'Content-Type: application/json' \
  --data-raw '{
    "uid": "M.B.6K_TN.0",
    "segment": "Manufacturing",
    "company": "6K Energy",
    "technology": "Batteries",
    "subcategory": "EAM",
    "investment_status": "U",
    "latitude": 35.606,
    "longitude": -88.83,
    "estimated_investment": 200438887,
    "announcement_date": "2023-04-18"
  }'
```