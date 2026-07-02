# sovereign-multiplicity

> C++ multiplicity functor for the SnapKitty ecosystem.
> Rational exponentiation. Overflow detection. Integer nth root.

[![License: Sovereign Source](https://img.shields.io/badge/License-Sovereign%20Source-blue.svg)](SOVEREIGN.md)
[![C++](https://img.shields.io/badge/C++-20-blue.svg)](https://isocpp.org/)
[![Tests](https://img.shields.io/badge/Tests-8%20passing-brightgreen.svg)](#testing)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    MULTIPLICITY FUNCTOR                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                    Input                                  │  │
│   │              base: uint64                                │  │
│   │              exponent: Rational64 (p/q)                  │  │
│   └──────────────────────────────────────────────────────────┘  │
│                           │                                      │
│                           ▼                                      │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                 Rational Reduction                        │  │
│   │         gcd(p, q) → simplified p'/q'                     │  │
│   └──────────────────────────────────────────────────────────┘  │
│                           │                                      │
│          ┌────────────────┼────────────────┐                    │
│          ▼                ▼                ▼                    │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐       │
│  │ Integer Exp  │ │  Nth Root    │ │  Overflow Check  │       │
│  │  (q=1 case)  │ │  (binary     │ │  (bit width)     │       │
│  │              │ │   search)    │ │                  │       │
│  └──────────────┘ └──────────────┘ └──────────────────┘       │
│          │                │                │                    │
│          └────────────────┼────────────────┘                    │
│                           ▼                                      │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                    Output                                 │  │
│   │              result: uint64                               │  │
│   │              overflow: bool                               │  │
│   └──────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Mathematical Definition

```
Multiplicity(base, exponent) where exponent ∈ Q

  exponent = p/q (Rational64)

  Cases:
    q = 1:  base^p          (integer exponent)
    q = 2:  √(base^p)       (square root)
    q = 3:  ∛(base^p)       (cube root)
    q = n:  ⁿ√(base^p)      (nth root)

  Negative exponent:
    base^(-p/q) = 1 / (base^(p/q))
```

## Quick Start

### Build

```bash
mkdir build && cd build
cmake ..
make -j$(nproc)
```

### Run Tests

```bash
./multiplicity_test
```

### API Usage

```cpp
#include "Multiplicity.h"

using namespace pirtm;

int main() {
    // Integer exponent
    Multiplicity m1(2, Rational64(3, 1));
    auto r1 = m1.compute();
    // r1.result = 8, r1.overflow = false

    // Square root
    Multiplicity m2(4, Rational64(1, 2));
    auto r2 = m2.compute();
    // r2.result = 2, r2.overflow = false

    // Cube root
    Multiplicity m3(27, Rational64(1, 3));
    auto r3 = m3.compute();
    // r3.result = 3, r3.overflow = false

    // Negative exponent
    Multiplicity m4(2, Rational64(-1, 1));
    auto r4 = m4.compute();
    // r4.result = 0, r4.overflow = false (integer division)

    // Overflow detection
    Multiplicity m5(UINT64_MAX, Rational64(2, 1));
    auto r5 = m5.compute();
    // r5.overflow = true
}
```

## Interactive Demo

```bash
# Demo 1: Integer exponent
$ ./multiplicity 2 3/1
2^(3/1) = 8

# Demo 2: Square root
$ ./multiplicity 4 1/2
4^(1/2) = 2

# Demo 3: Cube root
$ ./multiplicity 27 1/3
27^(1/3) = 3

# Demo 4: Overflow detection
$ ./multiplicity 18446744073709551615 2/1
Overflow detected!
```

## Implementation Details

| Feature | Implementation |
|---------|----------------|
| **Rational Type** | `Rational64 { p: i64, q: i64 }` with automatic reduction |
| **GCD** | Euclidean algorithm for rational reduction |
| **Integer Exponent** | Binary exponentiation (O(log n)) |
| **Nth Root** | Binary search with Newton refinement |
| **Overflow Check** | Bit width analysis before computation |

## Invariants

| Invariant | Description |
|-----------|-------------|
| **Deterministic** | Same input → same output |
| **Overflow-Safe** | All operations check for overflow |
| **Canonical** | Rationals are always in reduced form |
| **No Recursion** | All algorithms are iterative |

## Testing

```bash
# Run all tests
./multiplicity_test

# Run with verbose output
./multiplicity_test --verbose
```

## License

Sovereign Source License — see [SOVEREIGN.md](SOVEREIGN.md)

---

```
SOVEREIGN-MULTIPLICITY-001
Base. Exponent. Reduce. Compute. Verify.
Same input. Same output.
No recursion. No borrowed thesis.
```


---

## Citation

If you use this work, please cite:

```bibtex
@misc{snapkittywest2026sovereigncompute,
  title = {SNAPKITTYWEST: Sovereign Compute Architecture with Linear Types, WORM Seals, and Goldilocks Field Arithmetic},
  author = {SnapKitty Collective},
  year = {2026},
  doi = {10.5281/zenodo.21132094},
  url = {https://doi.org/10.5281/zenodo.21132094}
}
```

**Paper:** https://doi.org/10.5281/zenodo.21132094
**ORCID:** https://orcid.org/0009-0006-1916-5245
