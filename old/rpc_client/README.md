# Rpc Client Implementations

## Notes
- **DO NOT** use ethereum_types `to_string()` methods. they truncate output, causing errors when the string is sent to JsonRPC's
