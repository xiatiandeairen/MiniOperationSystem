# Self-Driving Iterations 4–23 Roadmap

## ROI-Ordered Execution Plan

### Rounds 4–6: Code Correctness (quick wins, 10-15min each)
| # | Task | ROI | Risk |
|---|------|-----|------|
| 4 | Ring Buffer: fix misleading "lock-free" comment + frame dealloc safety | High | None |
| 5 | ProcFS: live content generation + FileStat timestamps from TSC | High | None |
| 6 | Scheduler: boost O(n²)→O(n) + error handling consistency pass | High | None |

### Rounds 7–9: Test Coverage (15-20min each)
| 7 | Unit tests: MLFQ scheduler (tick/boost/demote/stats) | High | None |
| 8 | Unit tests: RamFS + VFS path resolution | High | None |
| 9 | Unit tests: IPC MessageQueue (send/recv/full/empty) | High | None |

### Rounds 10–12: Trace System Completeness (20-30min each)
| 10 | IPC Message: add TraceContext field for cross-process tracing | Med | Struct size increase |
| 11 | Shell `trace export`: serialize ring buffer to serial as JSON | Med | Output size |
| 12 | Trace Viewer: load real kernel data via serial capture script | Med | None |

### Rounds 13–15: Shell Feature Polish (10-20min each)
| 13 | Shell `ls` shows sizes; `cat /proc/meminfo` returns live data | High | None |
| 14 | Framebuffer color support: errors=red, prompt=green, output=white | Med | None |
| 15 | Shell `trace tree`: hierarchical span display in console | Med | None |

### Rounds 16–18: Architecture Quality (20-30min each)
| 16 | Memory: implement translate_addr + unmap_page | Med | Page table complexity |
| 17 | Per-process FD table (skeleton + data structure) | Low | Refactor VFS |
| 18 | README: complete quickstart guide with screenshots | High | None |

### Rounds 19–20: Wrap-up
| 19 | Update project.md + CHANGELOG + AGENTS.md | High | None |
| 20 | Final scoring + sync main + process optimization summary | High | None |

## Sync Points
- After round 6: sync to main
- After round 9: sync to main
- After round 12: sync to main
- After round 15: sync to main
- After round 18: sync to main
- After round 20: final sync to main
