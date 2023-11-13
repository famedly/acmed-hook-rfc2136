# acmed hook rfc2136

A hook for [acmed](https://github.com/breard-r/acmed) to support the `DNS-01` Challenge via Dynamic DNS Updates ([rfc2136](https://www.rfc-editor.org/rfc/rfc2136))

## Usage

An [example configuration](./config.toml.sample) can be found in the root of this repository.
It is required you set a resolver and alteast one zone.

The following example assumes `_acme-challenge.some-host.example.org.` to have a `CNAME` record to somewhere inside the `acme.example.org.` zone

```toml
resolver = "1.1.1.1:53"

[zones."acme.example.org."] # trailing dot is important
primary_ns = "1.2.3.4:53"
tsig_name = "my-tsig-name"
tsig_key = "" #base64 encoded key, standard alphabet, padded
tsig_algorithm = "hmac-sha256"
```

You can set the challenge record using the `set` subcommand, and clean it up using `unset`

Additionally, any command requires three options to be specified:


* `-c`, `--config <CONFIG>`
* `-i`, `--identifier <IDENTIFIER>`
* `-p`, `--proof <PROOF>`

Now you can use it like the following:

``` bash
acmed-hook-rfc2136 \\
--config config.toml.sample \\
--identifier some-host.example.org \\
--proof meow \\
set
```

## Building

`cargo build --release`

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.


## License

AGPL-3.0-only, see [LICENSE.md](./LICENSE.md)
