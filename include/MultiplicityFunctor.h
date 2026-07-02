//===- MultiplicityFunctor.h - Rational Exponentiation Engine -*- C++ -*-===//
//
// Computes p^m where p is a prime (u64) and m is a rational number (Q).
// Non-recursive. Every computation produces a deterministic WORM seal.
//
//===----------------------------------------------------------------------===//

#ifndef MULTIPLICITY_FUNCTOR_H
#define MULTIPLICITY_FUNCTOR_H

#include <cstdint>
#include <string>
#include <optional>

namespace pirtm {

/// Rational number with exact arithmetic (reduced form).
struct Rational {
    int64_t numer;
    int64_t denom;

    Rational();
    Rational(int64_t n, int64_t d);

    /// Reduce to lowest terms.
    void reduce();

    /// Check if this rational is valid (denom != 0, reduced).
    bool is_valid() const;

    /// Convert to string representation.
    std::string to_string() const;

    bool operator==(const Rational& other) const;
    bool operator!=(const Rational& other) const;
};

/// Error types for multiplicity computation.
enum class MultiplicityError {
    None,
    Overflow,        // p^m exceeds u64 range
    ZeroDenominator, // rational has denominator 0
    InvalidPrime,    // prime < 2
    NegativeExponent // negative exponent not supported in this context
};

/// Result of a multiplicity computation.
struct MultiplicityResult {
    Rational value;
    MultiplicityError error;
    std::string error_msg;

    bool ok() const { return error == MultiplicityError::None; }
};

/// Compute p^m where p is a prime and m is a rational exponent.
/// Returns exact Rational64 or MultiplicityError.
///
/// Examples:
///   multiplicity(3, Rational{5, 1}) = 243/1  (3^5)
///   multiplicity(8, Rational{2, 3}) = 1/8     (8^(2/3) = (2^3)^(2/3) = 2^2 = 4, but 8=2^3, so 8^(2/3)=4)
///   multiplicity(2, Rational{64, 1}) = overflow
MultiplicityResult compute_multiplicity(uint64_t prime, const Rational& exponent);

/// Verify that a prime index is valid (>= 2).
bool is_valid_prime(uint64_t p);

/// Get the largest prime index that doesn't overflow for a given exponent.
uint64_t max_prime_for_exponent(const Rational& exponent);

/// Format a multiplicity error as a human-readable string.
std::string format_error(MultiplicityError err, uint64_t prime, const Rational& exponent);

} // namespace pirtm

#endif // MULTIPLICITY_FUNCTOR_H
