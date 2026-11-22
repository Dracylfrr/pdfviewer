# Performance

Local Windows x64 measurements from the 0.1.0 development preview. Times are single observations, without a warm/cold benchmark series.

| Measurement | Observed |
| --- | ---: |
| Index 10,000 small synthetic PDF-named files (debug Rust test) | 1.74 seconds |
| Count indexed rows in the wide fixture folder | 1.79 milliseconds |
| Optimized NSIS installer | 6.16 MiB |
| Standalone ZIP | 7.64 MiB |

A fresh release process with the fixture profile took 619 ms to expose a native window. This timing ends at window creation, before WebView content readiness. A 1,250-page synthetic fixture mounted two page canvases at the starting viewport.

The fixture measures directory traversal and metadata indexing. It does not measure PDF parsing or the WebView. WebView startup, idle CPU, whole-process memory, and large-PDF scroll latency still need isolated measurement.
