# Benchmarks
Benchmarks captured with Criterion v0.8.1

System Information:
- OS: Arch Linux (Linux 6.17.9-arch1-1)
- CPU: AMD Ryzen 7 5800X (16) @ 5.36 GHz
- RAM: 32 GB

## HTML Parser
Date Captured: 2025-12-09 <sub>(YYYY-MM-DD)</sub>

### deep.html
Size: 12.0 MiB (12,585,574 bytes)

A deeply nested HTML file

| /         | Lower bound | Estimate  | Upper bound  |
| --------- | ----------- | --------- | ------------ |
| R²        | 0.0000149   | 0.0000155 | 0.0000150    |
| Mean      | 71.458 ms   | 72.134 ms | 72.773 ms    |
| Std. Dev. | 2.8374 ms   | 3.3949 ms | 3.8130 ms    |
| Median    | 73.564 ms   | 73.787 ms | 73.971 ms    |
| MAD 	    | 449.54 µs   | 620.94 µs | 928.68 µs    |

### mixed.html
Size: 1.9 MiB (2,033,047 bytes)

A mix between deeply nested and wide HTML file

| /         | Lower bound | Estimate  | Upper bound  |
| --------- | ----------- | --------- | ------------ |
| R²        | 0.0181297   | 0.0188303 |	0.0181672    |
| Mean      | 11.602 ms   | 11.740 ms | 11.874 ms    |
| Std. Dev. | 644.61 µs   | 699.15 µs | 730.96 µs    |
| Median    | 12.124 ms   | 12.246 ms | 12.258 ms    |
| MAD       | 49.133 µs   | 85.196 µs | 415.89 µs    |

### flat.html
Size: 5.3 MiB (5,513,294 bytes)

A flat HTML file with many sibling elements

| /         | Lower bound | Estimate  | Upper bound  |
| --------- | ----------- | --------- | ------------ |
| R²        | 0.0066130   | 0.0068584 | 0.0066039    |
| Mean      | 148.53 ms   | 150.83 ms | 153.18 ms    |
| Std. Dev. | 10.735 ms   | 11.979 ms | 13.050 ms    |
| Median    | 145.76 ms   | 149.43 ms | 154.09 ms    |
| MAD       | 10.933 ms   | 13.371 ms | 17.961 ms    |
