GET http://127.0.0.1:33003/auth
Authorization: Basic R1VWN0Q4M1hrWTZyaXcqV2R3NjNsblIkZiZRJXdHb1ZAa2RAUUFrNnhTcE5DLS1ScSRydiQ3VGE3b28wIzRNYjp2WCNxckxvWUpVTnNYRzVaMnRLaG1HWnkhR3JraTBlcVhmWEFfbnMqTERfMDFNaVlpNTYlWExKZVhkMmhmRVpO
# Authorization: Basic GUV7D83XkY6riw*Wdw63lnR$f&Q%wGoV@kd@QAk6xSpNC--Rq$rv$7Ta7oo0#4Mb:vX#qrLoYJUNsXG5Z2tKhmGZy!Grki0eqXfXA_ns*LD_01MiYi56%XLJeXd2hfEZN
HTTP 200
[Captures]
token: jsonpath "$"

GET http://127.0.0.1:33003/
Authorization: Bearer {{token}}
HTTP 200
