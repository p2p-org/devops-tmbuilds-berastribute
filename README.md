## Berachain Distributor

Validators on Berachain generate BGT after they've proposed a block but must call the Distributor's `distributeFor`
function with proof from the consensus client

This utility helps to accomplish that task

### Usage

```shell
git clonne https://github.com/datskos/berastribute
cargo build --release
```

- Optionally, modify the config file to override the default timing intervals.
- Uses a keystore wallet to call the distribute function. To create a new keystore wallet:

```shell
mkdir keys
cast wallet new --password keys/
````

Sample Usage:

```shell
./target/release/berastribute distributor \
   --fee-recipient <ADDRESS> \
   --wss-url <wss_url> \
   --beacon-url <beacon_url> \
   --keystore <keystore_path> \
   [--backfill-blocks <blocks>] \
   [--fallback-mode]
Options
  --fee-recipient: The address of the fee recipient (optional).
  --wss-url: The WebSocket URL of the EL client
  --beacon-url: The URL of the CL client
  --keystore: The path to the keystore file.
  --backfill-blocks: The number of past blocks to backfill (optional).
  --fallback-mode: Enable fallback mode (optional).
```

Note: if `--fee-recipient <ADDRESS>` is not specified, this will run for every block. If you want to limit it to only
running for your own validator then specify the fee recipient to match your validator's fee recipient and berastribute
will only run for blocks where `block.coinbase = fee_recipient`

If `--fallback-mode` is enabled then berastribute waits for 30s before calling distribute (can be adjusted via
the `fallback_wait_interval` key in config.toml). After 30s, it checks if the block is still actionable for rewards and
only calls distribute if still actionable. The reasoning here is that another party may call `distributeFor` on your
behalf so with this you can wait until that potentially happens.

### License

berastribute is licensed under the Apache and MIT licenses

### Disclaimer

*This code is being provided as is. No guarantee, representation or warranty is being made, express or implied, as to
the
safety or correctness of the code. It has not been audited and as such there can be no assurance it will work as
intended, and users may experience delays, failures, errors, omissions or loss of transmitted information. Nothing in
this repo should be construed as investment advice or legal advice for any particular facts or circumstances and is not
meant to replace competent counsel. It is strongly advised for you to contact a reputable attorney in your jurisdiction
for any questions or concerns with respect thereto. Author is not liable for any use of the foregoing, and users should
proceed with caution and use at their own risk*