GET http://127.0.0.1:3000/auth
[BasicAuth]
access: secret
HTTP 200
[Captures]
token: jsonpath "$"

# create a new feed
POST http://127.0.0.1:3000/feeds
Authorization: Bearer {{token}}
{
    "title": "hnrss/newcomments",
    "link": "https://hnrss.org/newcomments",
    "fetch_old_items": false
}
HTTP 200

# fetch all feeds
GET http://127.0.0.1:3000/feeds
Authorization: Bearer {{token}}
HTTP 200
[Captures]
feed_id: jsonpath "$[0].id"

# fetch a feed by id
GET http://127.0.0.1:3000/feeds/{{feed_id}}
Authorization: Bearer {{token}}
HTTP 200

# delete a feed by id
DELETE http://127.0.0.1:3000/feeds/{{feed_id}}
Authorization: Bearer {{token}}
HTTP 200

# check the feed is deleted
GET http://127.0.0.1:3000/feeds
Authorization: Bearer {{token}}
HTTP 200
[Asserts]
body == "[]"
