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
Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE1MTYyMzkwMjIsImV4cCI6MTUxNjIzOTAyMn0.tJrElL9vX4Budo1oZ9VvaWzkDXQ8BO5Wc5Cp4WITSAo
HTTP 401

# valid keys
GET http://127.0.0.1:33003/auth
[BasicAuth]
GUV7D83XkY6riwkWdw63lnRtfkQ2wGoV1kd3QAk6xSpNCteRqsrvt7Ta7oo0t4Mb: vXtqrLoYJUNsXG5Z2tKhmGZy3Grki0eqXfXAtns2LDe01MiYi56kXLJeXd2hfEZN
HTTP 200
[Captures]
valid_token: jsonpath "$"

# valid token
GET http://127.0.0.1:33003/
Authorization: Bearer {{valid_token}}
HTTP 200
