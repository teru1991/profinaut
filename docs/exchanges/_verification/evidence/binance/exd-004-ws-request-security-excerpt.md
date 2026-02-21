  * By default, an API key cannot `TRADE`. You need to enable trading in API Management first.

Security type |  Description
------------- | ------------
`NONE`        | Public market data
`TRADE`       | Trading on the exchange, placing and canceling orders
`USER_DATA`   | Private account information, such as order status and your trading history
`USER_STREAM` | Managing User Data Stream subscriptions

### SIGNED request security

* `SIGNED` requests require an additional parameter: `signature`, authorizing the request.

#### Signature Case Sensitivity

* **HMAC:** Signatures generated using HMAC are **not case-sensitive**. This means the signature string can be verified regardless of letter casing.
* **RSA:** Signatures generated using RSA are **case-sensitive**.
* **Ed25519:** Signatures generated using ED25519 are also **case-sensitive**

Please consult [SIGNED request example (HMAC)](#signed-request-example-hmac), [SIGNED request example (RSA)](#signed-request-example-rsa), and [SIGNED request example (Ed25519)](#signed-request-example-ed25519) on how to compute signature, depending on which API key type you are using.

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

**It is recommended to use a small `recvWindow` of 5000 or less!**

### SIGNED request example (HMAC)

Here is a step-by-step guide on how to sign requests using an HMAC secret key.

Example API key and secret key:

Key          | Value
------------ | ------------
`apiKey`       | `vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A`
`secretKey`    | `NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j`

**WARNING: DO NOT SHARE YOUR API KEY AND SECRET KEY WITH ANYONE.**

The example keys are provided here only for illustrative purposes.

Example of request with a symbol name comprised entirely of ASCII characters:

```json
{
    "id": "4885f793-e5ad-4c3b-8f6c-55d891472b71",
    "method": "order.place",
    "params": {
        "symbol": "BTCUSDT",
        "side": "SELL",
        "type": "LIMIT",
        "timeInForce": "GTC",
        "quantity": "0.01000000",
        "price": "52000.00",
        "recvWindow": 100,
        "timestamp": 1645423376532,
        "apiKey": "vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A",
        "signature": "------ FILL ME ------"
    }
}
```

Example of a request with a symbol name containing non-ASCII characters:

```json
{
    "id": "4885f793-e5ad-4c3b-8f6c-55d891472b71",
    "method": "order.place",
    "params": {
        "symbol": "１２３４５６",
        "side": "BUY",
        "type": "LIMIT",
        "timeInForce": "GTC",
        "quantity": "0.01000000",
        "price": "0.10000000",
        "recvWindow": 5000,
        "timestamp": 1645423376532,
        "apiKey": "vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A",
        "signature": "------ FILL ME ------"
    }
}
```

As you can see, the `signature` parameter is currently missing.

**Step 1: Construct the signature payload**

Take all request `params` except `signature` and **sort them in alphabetical order by parameter name**:

For the first set of example parameters (ASCII only):

Parameter        | Value
---------------- | ------------
`apiKey`           | vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A
`price`            | 52000.00
`quantity`         | 0.01000000
`recvWindow`       | 100
`side`             | SELL
`symbol`           | BTCUSDT
`timeInForce`      | GTC
`timestamp`        | 1645423376532
`type`             | LIMIT

For the second set of example parameters (some non-ASCII characters):

Parameter | Value
------------ | ------------
`apiKey` | vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A
`price` | 0.10000000
`quantity` | 1.00000000
`recvWindow` | 5000
`side` | BUY
`symbol` | １２３４５６
`timeInForce` | GTC
`timestamp` | 1645423376532
`type` | LIMIT

Format parameters as `parameter=value` pairs separated by `&`. Values need to be encoded in UTF-8.

For the first set of example parameters (ASCII only), the signature payload should look like this:

```console
apiKey=vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A&price=52000.00&quantity=0.01000000&recvWindow=100&side=SELL&symbol=BTCUSDT&timeInForce=GTC&timestamp=1645423376532&type=LIMIT
```

For the second set of example parameters (some non-ASCII characters), the signature payload should look like this:

```console
apiKey=vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A&price=0.10000000&quantity=1.00000000&recvWindow=5000&side=BUY&symbol=１２３４５６&timeInForce=GTC&timestamp=1645423376532&type=LIMIT
```

**Step 2: Compute the signature**

1. Use the `secretKey` of your API key as the signing key for the HMAC-SHA-256 algorithm.
2. Sign the UTF-8 bytes of the signature payload constructed in Step 1.
3. Encode the HMAC-SHA-256 output as a hex string.

Note that `apiKey`, `secretKey`, and the payload are **case-sensitive**, while the resulting signature value is case-insensitive.

You can cross-check your signature algorithm implementation with OpenSSL:

For the first set of example parameters (ASCII only):

```console
$ echo -n 'apiKey=vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A&price=52000.00&quantity=0.01000000&recvWindow=100&side=SELL&symbol=BTCUSDT&timeInForce=GTC&timestamp=1645423376532&type=LIMIT' \
  | openssl dgst -hex -sha256 -hmac 'NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j'

aa1b5712c094bc4e57c05a1a5c1fd8d88dcd628338ea863fec7b88e59fe2db24
```

For the second set of example parameters (some non-ASCII characters):

```console
$ echo -n 'apiKey=vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A&price=0.10000000&quantity=1.00000000&recvWindow=5000&side=BUY&symbol=１２３４５６&timeInForce=GTC&timestamp=1645423376532&type=LIMIT' \
  | openssl dgst -hex -sha256 -hmac 'NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j'

b33892ae8e687c939f4468c6268ddd4c40ac1af18ad19a064864c47bae0752cd
```

**Step 3: Add `signature` to request `params`**

Complete the request by adding the `signature` parameter with the signature string.

For the first set of example parameters (ASCII only):

```json
{
    "id": "4885f793-e5ad-4c3b-8f6c-55d891472b71",
    "method": "order.place",
    "params": {
        "symbol": "BTCUSDT",
        "side": "SELL",
        "type": "LIMIT",
        "timeInForce": "GTC",
        "quantity": "0.01000000",
        "price": "52000.00",
        "recvWindow": 100,
        "timestamp": 1645423376532,
        "apiKey": "vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A",
        "signature": "aa1b5712c094bc4e57c05a1a5c1fd8d88dcd628338ea863fec7b88e59fe2db24"
    }
}
```

For the second set of example parameters (some non-ASCII characters):

```json
{
    "id": "4885f793-e5ad-4c3b-8f6c-55d891472b71",
    "method": "order.place",
    "params": {
