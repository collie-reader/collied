GET http://127.0.0.1:33003/auth
[BasicAuth]
access: secret
HTTP 200
[Captures]
token: jsonpath "$"

# create a new feed
POST http://127.0.0.1:33003/feeds
Authorization: Bearer {{token}}
{
    "title": "hnrss/newcomments",
    "link": "https://hnrss.org/newcomments",
    "fetch_old_items": true
}
HTTP 200

# fetch the all feeds
GET http://127.0.0.1:33003/feeds
Authorization: Bearer {{token}}
HTTP 200
