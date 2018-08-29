# Absentis
A Project for Quickblocks bounty that finds missing transactions



An Example Configuration file:

```toml

[[nodes]]
identifier = 'ParityLocal'
transport = 'Http'

[nodes.http]
url = 'http://localhost'
port = 8545

[nodes.ipc]
path = '/home/me/parity/ipc'

[[nodes]]
identifier = 'GethRemoteNode'
transport = 'Http'

[nodes.http]
url = 'http://32.0.1.32'
port = 8545

```

