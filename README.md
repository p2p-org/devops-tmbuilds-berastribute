## Berachain Distributor

Validators on Berachain generate BGT after they've proposed a block but must call the Distributor's `distributeFor`
function with proof from the consensus client

This utility helps to accomplish that task

### Usage

```shell
cargo build --release
```

- Optionally, modify the config file to override the default timing intervals.
- Uses a keystore wallet to call the distribute function. Can create:  `cast wallet new --password keys/`

Sample Usage:

```shell
./target/release/berastribute distributor -f <YOUR_FEE_RECIPIENT> --keystore <KEYSTORE_PATH> --wss-url ws://localhost:8546 -b http://localhost:3500
```

### Disclaimer

This code is being provided as is. No guarantee, representation or warranty is being made, express or implied, as to the
safety or correctness of the code. It has not been audited and as such there can be no assurance it will work as
intended, and users may experience delays, failures, errors, omissions or loss of transmitted information. Nothing in
this repo should be construed as investment advice or legal advice for any particular facts or circumstances and is not
meant to replace competent counsel. It is strongly advised for you to contact a reputable attorney in your jurisdiction
for any questions or concerns with respect thereto. Author is not liable for any use of the foregoing, and users should
proceed with caution and use at their own risk..