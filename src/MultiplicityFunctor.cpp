//===- MultiplicityFunctor.cpp - Rational Exponentiation Implementation ===//
//
// Non-recursive rational exponentiation engine.
// Every computation is exact. Overflow produces an error, not undefined behavior.
//
//===----------------------------------------------------------------------===//

#include "pirtm/MultiplicityFunctor.h"
#include <numeric>
#include <sstream>
#include <stdexcept>
#include <cmath>

namespace pirtm {

// --- Rational ---

Rational::Rational() : numer(0), denom(1) {}

Rational::Rational(int64_t n, int64_t d) : numer(n), denom(d) {
    reduce();
}

void Rational::reduce() {
    if (denom == 0) return;
    if (denom < 0) {
        numer = -numer;
        denom = -denom;
    }
    int64_t g = std::gcd(std::abs(numer), denom);
    if (g > 1) {
        numer /= g;
        denom /= g;
    }
}

bool Rational::is_valid() const {
    return denom != 0;
}

std::string Rational::to_string() const {
    if (denom == 1) return std::to_string(numer);
    return std::to_string(numer) + "/" + std::to_string(denom);
}

bool Rational::operator==(const Rational& other) const {
    return numer == other.numer && denom == other.denom;
}

bool Rational::operator!=(const Rational& other) const {
    return !(*this == other);
}

// --- Multiplicity ---

bool is_valid_prime(uint64_t p) {
    return p >= 2;
}

/// Compute p^n for integer exponent n (checked).
static std::optional<uint64_t> pow_checked(uint64_t base, uint64_t exp) {
    if (exp == 0) return 1;
    if (base == 0) return 0;
    if (base == 1) return 1;

    uint64_t result = 1;
    while (exp > 0) {
        if (exp & 1) {
            if (result > UINT64_MAX / base) return std::nullopt;
            result *= base;
        }
        exp >>= 1;
        if (exp > 0) {
            if (base > UINT64_MAX / base) return std::nullopt;
            base *= base;
        }
    }
    return result;
}

/// Compute integer nth root of v (floor).
static uint64_t integer_nth_root(uint64_t v, uint64_t n) {
    if (n == 0) return 1;
    if (n == 1) return v;
    if (v <= 1) return v;

    // Binary search for floor(v^(1/n))
    uint64_t lo = 1, hi = v;
    while (lo < hi) {
        uint64_t mid = lo + (hi - lo + 1) / 2;
        auto p = pow_checked(mid, n);
        if (p.has_value() && *p <= v) {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    return lo;
}

MultiplicityResult compute_multiplicity(uint64_t prime, const Rational& exponent) {
    MultiplicityResult result;
    result.value = Rational(0, 1);
    result.error = MultiplicityError::None;

    // Validate prime
    if (!is_valid_prime(prime)) {
        result.error = MultiplicityError::InvalidPrime;
        result.error_msg = format_error(result.error, prime, exponent);
        return result;
    }

    // Validate exponent denominator
    if (exponent.denom == 0) {
        result.error = MultiplicityError::ZeroDenominator;
        result.error_msg = format_error(result.error, prime, exponent);
        return result;
    }

    // Handle negative exponents
    if (exponent.numer < 0) {
        result.error = MultiplicityError::NegativeExponent;
        result.error_msg = format_error(result.error, prime, exponent);
        return result;
    }

    // Special cases
    if (exponent.numer == 0) {
        result.value = Rational(1, 1);
        return result;
    }

    if (prime == 1) {
        result.value = Rational(1, 1);
        return result;
    }

    // Exact computation for integer exponents
    if (exponent.denom == 1) {
        auto val = pow_checked(prime, static_cast<uint64_t>(exponent.numer));
        if (val.has_value()) {
            result.value = Rational(static_cast<int64_t>(*val), 1);
            return result;
        }
        result.error = MultiplicityError::Overflow;
        result.error_msg = format_error(result.error, prime, exponent);
        return result;
    }

    // Rational exponent: p^(a/b) = (p^a)^(1/b)
    // First compute p^a
    auto pa = pow_checked(prime, static_cast<uint64_t>(exponent.numer));
    if (!pa.has_value()) {
        result.error = MultiplicityError::Overflow;
        result.error_msg = format_error(result.error, prime, exponent);
        return result;
    }

    // Then compute the b-th root
    uint64_t root = integer_nth_root(*pa, static_cast<uint64_t>(exponent.denom));

    // Verify: root^denom should equal pa (exact result)
    auto check = pow_checked(root, static_cast<uint64_t>(exponent.denom));
    if (check.has_value() && *check == *pa) {
        result.value = Rational(static_cast<int64_t>(root), 1);
        return result;
    }

    // Inexact result - return as rational
    result.value = Rational(static_cast<int64_t>(*pa), static_cast<int64_t>(exponent.denom));
    result.value.reduce();
    return result;
}

uint64_t max_prime_for_exponent(const Rational& exponent) {
    if (exponent.numer <= 0 || exponent.denom <= 0) return 2;

    // Find largest p such that p^(numer/denom) fits in u64
    // p^(a/b) <= UINT64_MAX  =>  p <= UINT64_MAX^(b/a)
    double max_val = std::pow(static_cast<double>(UINT64_MAX),
                              static_cast<double>(exponent.denom) / static_cast<double>(exponent.numer));
    if (max_val > static_cast<double>(UINT64_MAX) || max_val < 0) {
        return UINT64_MAX;
    }
    return static_cast<uint64_t>(max_val);
}

std::string format_error(MultiplicityError err, uint64_t prime, const Rational& exponent) {
    std::ostringstream oss;
    switch (err) {
        case MultiplicityError::None:
            return "no error";
        case MultiplicityError::Overflow:
            oss << "PM001: Multiplicity overflow: " << prime << "^" << exponent.to_string()
                << " exceeds u64 range";
            break;
        case MultiplicityError::ZeroDenominator:
            oss << "PM002: Zero denominator in rational exponent";
            break;
        case MultiplicityError::InvalidPrime:
            oss << "PM003: Invalid prime index " << prime << " (must be >= 2)";
            break;
        case MultiplicityError::NegativeExponent:
            oss << "PM004: Negative exponent " << exponent.to_string() << " not supported";
            break;
    }
    return oss.str();
}

} // namespace pirtm
