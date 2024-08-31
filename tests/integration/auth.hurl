# invalid keys
GET http://127.0.0.1:33003/auth
[BasicAuth]
invalid_access: invalid_secret
HTTP 401

# invalid token
GET http://127.0.0.1:33003/
Authorization: Bearer invalid_token
HTTP 401

# expired token
GET http://127.0.0.1:33003/
Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE1MTYyMzkwMjIsImV4cCI6MTUxNjIzOTAyMn0.Zny-oK4D-3ctQmpfzJT-s_KmQYdgq_3ZucOVXdtwoVY
HTTP 401

# valid keys
GET http://127.0.0.1:33003/auth
[BasicAuth]
access: secret
HTTP 200
[Captures]
valid_token: jsonpath "$"

# valid token
GET http://127.0.0.1:33003/
Authorization: Bearer {{valid_token}}
HTTP 200
