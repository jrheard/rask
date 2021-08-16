[![codecov](https://codecov.io/gh/jrheard/rask/branch/main/graph/badge.svg?token=BAZT2L4F24)](https://codecov.io/gh/jrheard/rask) (rask_cli integration test coverage isn't picked up by tarpaulin, [known issue](https://github.com/xd009642/tarpaulin/issues/616).)

See [design-doc.md](design-doc.md) for more info.


Development
===========

To run a local `rask_api` server that restarts whenever the code is changed:

```
ROCKET_PORT=8001 cargo watch -x 'run --bin rask_api'
```

Testing
=======

```
docker-compose -f docker-compose-test.yaml up -d --build
cargo test -- --test-threads=1
```
