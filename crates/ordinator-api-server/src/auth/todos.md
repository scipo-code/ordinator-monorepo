So I have:
-[x] impl AuthError
-[x] TokenClaims
-[x] RefreshTokenClaims
-[x] three traits for implementing provider authentication
-[x] a JwtConfig for setting up the secrets, expiration etc.
-[ ] A AuthPayload which is not correct.. Or rather it only works for local authentication.
	- Do I need a separate type of payload for external providers?


I need to impl:
-[x] For starters a simple database to set up the login.
-[ ] I need to set up the /auth/authorize and /auth/refresh
-[ ] Create the middleware to protect all the other routes.
-[ ] Update my frontend apis to use the tokens. I will need to create a class Api in the frontend
and refactor.

