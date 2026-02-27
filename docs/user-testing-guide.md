# MiniOS User Testing Guide

## For Test Participants

Thank you for helping test MiniOS! Follow these steps:

### Setup (2 minutes)
1. Clone the repo: `git clone <url>`
2. Run: `cargo make run-gui`
3. Wait ~15 seconds for boot

### Testing Tasks (15 minutes)

**Task 1: First Impressions** (3 min)
- Type `tutorial` — does the guide make sense?
- Rate clarity: 1-5

**Task 2: Learning Path** (5 min)
- Follow the tutorial steps 1-5
- Type `journey` — is the progress tracking intuitive?
- Rate: 1-5

**Task 3: Deep Dive** (5 min)
- Run `explain ls` — does the explanation help you understand?
- Run `trace follow ls /` — is the trace output useful?
- Run `compare scheduler` — is the comparison clear?
- Rate usefulness: 1-5

**Task 4: Experiments** (2 min)
- Run `lab memory-usage` — did you learn something?
- Run `crash oom` — was it educational?
- Rate: 1-5

### Feedback Form
After testing, answer these questions:
1. What was the most valuable thing you learned?
2. What was confusing or unclear?
3. What feature would you add?
4. Would you recommend this to a friend learning OS? (1-5)

Email feedback to: [project maintainer]
