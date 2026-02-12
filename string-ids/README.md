# oxide-string-ids

Brute force script to recover string IDs from hashes .dat locale files.

## Prerequisites

* Node.js

## Usage

```bash
# To brute force the default range
node ./generate.js ./en_us_data.dat out.txt
# To brute force string hashes between 0 and 10,000,000
node ./generate.js ./en_us_data.dat out.txt 0 10000000
```

## Known Issues

There are more efficient ways to recover the string IDs, even while keeping the simple brute force approach.
