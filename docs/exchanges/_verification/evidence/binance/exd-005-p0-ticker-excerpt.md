### Symbol price ticker
```
GET /api/v3/ticker/price
```
Latest price for a symbol or symbols.

**Weight:**

<table>
<thead>
    <tr>
        <th>Parameter</th>
        <th>Symbols Provided</th>
        <th>Weight</th>
    </tr>
</thead>
<tbody>
    <tr>
        <td rowspan="2">symbol</td>
        <td>1</td>
        <td>2</td>
    </tr>
    <tr>
        <td>symbol parameter is omitted</td>
        <td>4</td>
    </tr>
    <tr>
        <td>symbols</td>
        <td>Any</td>
        <td>4</td>
    </tr>
</tbody>
</table>

**Parameters:**

<table>
<thead>
    <tr>
      <th>Name</th>
      <th>Type</th>
      <th>Mandatory</th>
      <th>Description</th>
    </tr>
</thead>
<tbody>
    <tr>
        <td>symbol</td>
        <td>STRING</td>
        <td>NO</td>
        <td rowspan="2"> Parameter symbol and symbols cannot be used in combination. <br/> If neither parameter is sent, prices for all symbols will be returned in an array. <br/><br/>
        Examples of accepted format for the symbols parameter:
         ["BTCUSDT","BNBUSDT"] <br/>
         or <br/>
         %5B%22BTCUSDT%22,%22BNBUSDT%22%5D</td>
    </tr>
    <tr>
        <td>symbols</td>
        <td>STRING</td>
        <td>NO</td>
    </tr>
    <tr>
        <td>symbolStatus</td>
        <td>ENUM</td>
        <td>NO</td>
        <td>Filters for symbols that have this <code>tradingStatus</code>.<br>For a single symbol, a status mismatch returns error <code>-1220 SYMBOL_DOES_NOT_MATCH_STATUS</code>. <br>For multiple or all symbols, non-matching ones are simply excluded from the response.<br>Valid values: <code>TRADING</code>, <code>HALT</code>, <code>BREAK</code> </td>
    </tr>
</tbody>
</table>


**Data Source:**
Memory

**Response:**
```javascript
{
  "symbol": "LTCBTC",
  "price": "4.00000200"
}
```
OR
```javascript
[
  {
    "symbol": "LTCBTC",
    "price": "4.00000200"
  },
  {
    "symbol": "ETHBTC",
    "price": "0.07946600"
  }
]
```

### Symbol order book ticker
