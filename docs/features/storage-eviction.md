# Storage eviction
---
priority: mid
---

When automatic eviction is implemented, the proposed priority order (lowest to
highest protection) is:

1. **Evict first**: played episodes, no flag, oldest first
2. **Evict next**: unplayed episodes, oldest first
3. **Never evict**: flagged / starred episodes

All eviction rules should be user-configurable for power users.

