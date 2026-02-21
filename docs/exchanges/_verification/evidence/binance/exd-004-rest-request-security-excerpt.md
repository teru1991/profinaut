`USER_DATA`   | Private account information, such as order status and your trading history
`USER_STREAM` | Managing User Data Stream subscriptions

### SIGNED Endpoint security

* `SIGNED` endpoints require an additional parameter, `signature`, to be sent in the `query string` or `request body`.

#### Signature Case Sensitivity

* **HMAC:** Signatures generated using HMAC are **not case-sensitive**. This means the signature string can be verified regardless of letter casing.
* **RSA:** Signatures generated using RSA are **case-sensitive**.
* **Ed25519:** Signatures generated using Ed25519 are also **case-sensitive**

Please consult [SIGNED request example (HMAC)](#hmac-keys), [SIGNED request example (RSA)](#rsa-keys), and [SIGNED request example (Ed25519)](#ed25519-keys) on how to compute signature, depending on which API key type you are using.

<a id="timingsecurity"></a>

### Timing security
* `SIGNED` requests also require a `timestamp` parameter which should be the current timestamp either in milliseconds or microseconds. (See [General API Information](#general-api-information))
* An additional optional parameter, `recvWindow`, specifies for how long the request stays valid and may only be specified in milliseconds.
  * `recvWindow` supports up to three decimal places of precision (e.g., 6000.346) so that microseconds may be specified.
  * If `recvWindow` is not sent, **it defaults to 5000 milliseconds**.
  * Maximum `recvWindow` is 60000 milliseconds.
* Request processing logic is as follows:

```javascript
serverTime = getCurrentTime()
if (timestamp < (serverTime + 1 second) && (serverTime - timestamp) <= recvWindow) {
  // begin processing request
  serverTime = getCurrentTime()
  if (serverTime - timestamp) <= recvWindow {
    // forward request to Matching Engine
  } else {
    // reject request
  }
  // finish processing request
} else {
  // reject request
}
```

**Serious trading is about timing.** Networks can be unstable and unreliable,
which can lead to requests taking varying amounts of time to reach the
servers. With `recvWindow`, you can specify that the request must be
processed within a certain number of milliseconds or be rejected by the
server.


**It is recommended to use a small recvWindow of 5000 or less! The max cannot go beyond 60,000!**

### SIGNED Endpoint Examples for POST /api/v3/order

#### HMAC Keys

The signature payload of your request is the query string concatenated without separator to the HTTP body. Any non-ASCII character must be percent-encoded before signing.

Here is a step-by-step example of how to send a valid signed payload from the Linux command line using `echo`, `openssl`, and `curl`. There is one example with a symbol name comprised entirely of ASCII characters and one example with a symbol name containing non-ASCII characters.

Example API key and secret key:

Key | Value
------------ | ------------
`apiKey` | vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A
`secretKey` | NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j

**WARNING: DO NOT SHARE YOUR API KEY AND SECRET KEY WITH ANYONE.**

The example keys are provided here only for illustrative purposes.

Example of request with a symbol name comprised entirely of ASCII characters:

Parameter | Value
------------ | ------------
`symbol` | LTCBTC
`side` | BUY
`type` | LIMIT
`timeInForce` | GTC
`quantity` | 1
`price` | 0.1
`recvWindow` | 5000
`timestamp` | 1499827319559

Example of a request with a symbol name containing non-ASCII characters:

Parameter | Value
------------ | ------------
`symbol` | １２３４５６
`side` | BUY
`type` | LIMIT
`timeInForce` | GTC
`quantity` | 1
`price` | 0.1
`recvWindow` | 5000
`timestamp` | 1499827319559

**Step 1: Construct the signature payload**

1. Format parameters as `parameter=value` pairs separated by `&`.
2. Percent-encode the string.

For the first set of example parameters (ASCII only), the `parameter=value` string should look like this:

```console
symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559
```

After percent-encoding, the signature payload should look like this:

```console
symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559
```

For the second set of example parameters (some non-ASCII characters), the `parameter=value` string should look like this:

```console
symbol=１２３４５６&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559
```

After percent-encoding, the signature payload should look like this:

```console
symbol=%EF%BC%91%EF%BC%92%EF%BC%93%EF%BC%94%EF%BC%95%EF%BC%96&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559
```

**Step 2: Compute the signature**

1. Use the `secretKey` of your API key as the signing key for the HMAC-SHA-256 algorithm.
2. Sign the signature payload constructed in Step 1.
3. Encode the HMAC-SHA-256 output as a hex string.

Note that `secretKey` and the payload are **case-sensitive**, while the resulting signature value is case-insensitive.

**Example commands**

For the first set of example parameters (ASCII only):

```console
$ echo -n "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559" | openssl dgst -sha256 -hmac "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j"

c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71
```

For the second set of example parameters (some non-ASCII characters):

```console
$ echo -n "symbol=%EF%BC%91%EF%BC%92%EF%BC%93%EF%BC%94%EF%BC%95%EF%BC%96&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559" | openssl dgst -sha256 -hmac "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j"

e1353ec6b14d888f1164ae9af8228a3dbd508bc82eb867db8ab6046442f33ef3
```

**Step 3: Add signature to the request**

Complete the request by adding the `signature` parameter to the query string.

For the first set of example parameters (ASCII only):

```console
curl -s -v -H "X-MBX-APIKEY: $apiKey" -X POST "https://api.binance.com/api/v3/order?symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559&signature=c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71"
```

For the second set of example parameters (some non-ASCII characters)

```console
curl -s -v -H "X-MBX-APIKEY: $apiKey" -X POST "https://api.binance.com/api/v3/order?symbol=%EF%BC%91%EF%BC%92%EF%BC%93%EF%BC%94%EF%BC%95%EF%BC%96&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559&signature=e1353ec6b14d888f1164ae9af8228a3dbd508bc82eb867db8ab6046442f33ef3"
```

Here is a sample Bash script performing all the steps above:

```bash
apiKey="vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A"
secretKey="NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j"

payload="symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559"

# Sign the request

signature=$(echo -n "$payload" | openssl dgst -sha256 -hmac "$secretKey")
signature=${signature#*= }    # Keep only the part after the "= "

# Send the request

