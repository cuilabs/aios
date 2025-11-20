# AIOS SDKs

SDKs for developing on AIOS.

## Rust SDK

Kernel and userland development SDK.

```rust
use aios_sdk::*;

let kernel = KernelClient::new();
let memory = kernel.allocate_memory(1024)?;
```

## TypeScript SDK

Userland service development SDK.

```typescript
import { KernelClient, MemoryFabricClient } from "@aios/sdk-typescript";

const kernel = new KernelClient();
const memory = await kernel.allocateMemory(1024);
```

## Python SDK

Agent development SDK.

```python
from aios import KernelClient, AgentClient

kernel = KernelClient()
agent = AgentClient()
agent_id = agent.spawn({"memorySize": 1024})
```

