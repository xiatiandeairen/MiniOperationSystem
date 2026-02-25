# Risk Register

| ID | Risk | Probability | Impact | Status | Mitigation | Checkpoint |
|----|------|-------------|--------|--------|------------|------------|
| R01 | bootloader_api version incompatibility | Medium | High | 🟢 Open | Pin version; test at CP2 | CP2 |
| R02 | Context-switch assembly correctness | High | High | 🟢 Open | Reference xv6/BlogOS; extensive QEMU testing | CP6 |
| R03 | Ring buffer concurrency bugs | Medium | High | 🟢 Open | Atomic-only writes; stress tests | CP3 |
| R04 | Memory management instability | Medium | High | 🟢 Open | Incremental testing per allocation layer | CP4 |
| R05 | MLFQ scheduler starvation | Low | Medium | 🟢 Open | Periodic priority boost | CP6 |
