# Authentication

The Fuel Indexer service functionality offers users a range of options for verifying their identity. The system supports any arbitrary authentication scheme (in theory), although it in practice the service defaults to JWT authentication due to its stateless nature and popularity. To authenticate using JWT, users ask an index operator for a nonce, sign that nonce with their wallet, then send the signature and the nonce to the indexer operator for verification. Once the signature is confirmed as valid, a valid JWT is produced and returned to the user, and the user is authenticated.

It is important to note that authentication is disabled by default. However, if authentication is enabled, users will need to authenticate before performing operations that involve modifying the state of the service, such as uploading or stopping indexers, etc. The new authentication functionality offers a flexible and secure way for users to authenticate and perform operations that affect the service's state.

## Usage

Below is a demonstration of basic JWT authentication using an indexer operator at https://index.swayswap.io

```bash
forc index auth --account 0 --url https://index.swayswap.io:29987
```

You will first be prompted for the password for your wallet:

```text
Please enter your password:
```

After successfully entering your wallet password you should be presented with your new JWT token.


```text
✅ Successfully authenticated at https://index.swayswap.io:29987/api/auth/signature.

Token: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiODNlNjhiOTFmNDhjYWM4M....
```

Use this token in your `Authorization` headers when making requests for operations such as uploading indexers, stopping indexers, and other operations that mutate state in this way.
