# Absentis
A Project for Quickblocks bounty that finds missing transactions



An Example Configuration file:

```toml

[[nodes]]
type = 'Parity'

[nodes.http]
url = 'http://localhost'
port = 8545

[[nodes]]
type = 'Geth'

[nodes.http]
url = 'http://localhost'
port = 8545

[nodes.ipc]
path = '/home/insi/some_ipc'

```
