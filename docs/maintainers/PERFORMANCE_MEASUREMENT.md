# Performance Measurement

**Status:** Maintenance scaffold for repeatable local measurements. This is
not a completed performance pass, release threshold, or supported-platform
result.

## Current measurement

The editor benchmark starts and destroys the current Tiptap extension set with
a fixed 1,000-paragraph document. The fixture is created before timing begins,
then deeply frozen so no iteration can modify shared input. Each timed
iteration creates a new editor and destroys it before returning. The result
therefore includes both editor setup and teardown, but not fixture construction.

Run it with:

```bash
npm run bench:editor
```

The benchmark is manual. Normal `npm test` and `scripts/verify.sh` runs exclude
`.bench.ts` files unless the benchmark command is intentionally invoked.

The benchmark has no pass or fail threshold. Use it to compare nearby changes
on the same machine, operating system, Node.js version, and dependency lock.
Record that environment with any result used in a review. Do not commit raw
benchmark output as a product performance claim.

## Limits

Vitest runs this measurement in jsdom. It does not measure Tauri startup,
platform WebView rendering, native window creation, filesystem work, or a
packaged desktop build. Results from different machines are not directly
comparable.

Release-level performance evidence still requires supported-platform desktop
measurement and separate coverage for large reference libraries and export
workloads. Those measurements should be added only around existing behavior,
without changing product code to improve an unverified number.
