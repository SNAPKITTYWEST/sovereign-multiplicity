//! # sovereign-multiplicity
//!
//! Quantum algorithms: QFT, Grover, Shor, QPE.

pub mod core {
//! # utqc-core
//!
//! Circuit IR — Gate, Qubit, Circuit, Measurement.
//! Non-recursive. Every circuit compiles to a flat list of operations.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors in circuit construction or execution.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CircuitError {
    /// Qubit index out of bounds.
    #[error("qubit index {0} out of bounds (circuit has {1} qubits)")]
    QubitOutOfBounds(usize, usize),

    /// Duplicate measurement on the same qubit.
    #[error("duplicate measurement on qubit {0}")]
    DuplicateMeasurement(usize),

    /// Empty circuit.
    #[error("circuit is empty")]
    EmptyCircuit,
}

/// A single qubit identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Qubit(pub usize);

/// Single-qubit gate types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SingleGate {
    /// Pauli-X (NOT).
    PauliX,
    /// Pauli-Y.
    PauliY,
    /// Pauli-Z.
    PauliZ,
    /// Hadamard.
    Hadamard,
    /// T-gate (π/8 phase).
    TGate,
    /// S-gate (π/4 phase).
    SGate,
}

/// Two-qubit gate types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DoubleGate {
    /// Controlled-NOT.
    CNOT,
    /// Controlled-Z.
    CZ,
    /// SWAP.
    SWAP,
}

/// A gate operation in the circuit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Gate {
    /// Single-qubit gate.
    Single {
        /// Gate type.
        gate: SingleGate,
        /// Target qubit.
        target: Qubit,
    },
    /// Two-qubit gate.
    Double {
        /// Gate type.
        gate: DoubleGate,
        /// Control qubit.
        control: Qubit,
        /// Target qubit.
        target: Qubit,
    },
    /// Rotation gate (parameterized).
    Rotation {
        /// Target qubit.
        target: Qubit,
        /// Angle in radians.
        angle: f64,
    },
}

/// A measurement record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Measurement {
    /// Qubit being measured.
    pub qubit: Qubit,
    /// Classical bit index to store result.
    pub classical_bit: usize,
}

/// A quantum circuit — non-recursive flat IR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Circuit {
    /// Number of qubits in the circuit.
    pub num_qubits: usize,
    /// Number of classical bits.
    pub num_classical_bits: usize,
    /// Ordered list of gate operations.
    pub gates: Vec<Gate>,
    /// Measurements to perform at the end.
    pub measurements: Vec<Measurement>,
}

impl Circuit {
    /// Create a new empty circuit.
    pub fn new(num_qubits: usize, num_classical_bits: usize) -> Self {
        Self {
            num_qubits,
            num_classical_bits,
            gates: Vec::new(),
            measurements: Vec::new(),
        }
    }

    /// Add a gate to the circuit.
    pub fn add_gate(&mut self, gate: Gate) -> Result<(), CircuitError> {
        match &gate {
            Gate::Single { target, .. } => {
                if target.0 >= self.num_qubits {
                    return Err(CircuitError::QubitOutOfBounds(target.0, self.num_qubits));
                }
            }
            Gate::Double { control, target, .. } => {
                if control.0 >= self.num_qubits {
                    return Err(CircuitError::QubitOutOfBounds(control.0, self.num_qubits));
                }
                if target.0 >= self.num_qubits {
                    return Err(CircuitError::QubitOutOfBounds(target.0, self.num_qubits));
                }
            }
            Gate::Rotation { target, .. } => {
                if target.0 >= self.num_qubits {
                    return Err(CircuitError::QubitOutOfBounds(target.0, self.num_qubits));
                }
            }
        }
        self.gates.push(gate);
        Ok(())
    }

    /// Add a measurement.
    pub fn add_measurement(&mut self, qubit: Qubit, classical_bit: usize) -> Result<(), CircuitError> {
        if qubit.0 >= self.num_qubits {
            return Err(CircuitError::QubitOutOfBounds(qubit.0, self.num_qubits));
        }
        if self.measurements.iter().any(|m| m.qubit == qubit) {
            return Err(CircuitError::DuplicateMeasurement(qubit.0));
        }
        self.measurements.push(Measurement { qubit, classical_bit });
        Ok(())
    }

    /// Number of gates in the circuit.
    pub fn depth(&self) -> usize {
        self.gates.len()
    }

    /// Validate the circuit.
    pub fn validate(&self) -> Result<(), CircuitError> {
        if self.gates.is_empty() && self.measurements.is_empty() {
            return Err(CircuitError::EmptyCircuit);
        }
        Ok(())
    }
}

/// The non-recursive pass trait.
pub trait Pass {
    /// Input type for this pass.
    type Input;
    /// Output type for this pass.
    type Output;

    /// Name of this pass.
    fn name(&self) -> &'static str;

    /// Execute the pass.
    fn run(&self, input: Self::Input) -> Result<Self::Output, CircuitError>;
}

}

pub mod goldilocks {
//! # utqc-goldilocks
//!
//! Goldilocks field arithmetic (p = 2^64 - 2^32 + 1).
//! Used in PLONK and other ZK-proof systems.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Goldilocks field error.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GoldilocksError {
    /// Division by zero.
    #[error("division by zero")]
    DivisionByZero,
}

/// Goldilocks field element (mod p = 2^64 - 2^32 + 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Goldilocks(pub u64);

impl Goldilocks {
    /// The Goldilocks prime.
    pub const P: u64 = 18_446_744_069_414_584_321;
    /// Zero element.
    pub const ZERO: Self = Self(0);
    /// One element.
    pub const ONE: Self = Self(1);

    /// Create a new element, reducing mod p.
    pub fn new(val: u64) -> Self {
        Self(Self::reduce(val))
    }

    /// Reduce mod p.
    fn reduce(val: u64) -> u64 {
        if val >= Self::P {
            val % Self::P
        } else {
            val
        }
    }

    /// Addition.
    pub fn add(self, other: Self) -> Self {
        let (s, overflow) = self.0.overflowing_add(other.0);
        let mut result = s;
        if overflow || result >= Self::P {
            result = result.wrapping_sub(Self::P);
        }
        Self(result)
    }

    /// Subtraction.
    pub fn sub(self, other: Self) -> Self {
        let (s, overflow) = self.0.overflowing_sub(other.0);
        if overflow {
            Self(s.wrapping_add(Self::P))
        } else {
            Self(s)
        }
    }

    /// Multiplication.
    pub fn mul(self, other: Self) -> Self {
        let result = (self.0 as u128) * (other.0 as u128);
        Self((result % Self::P as u128) as u64)
    }

    /// Power.
    pub fn pow(self, exp: u64) -> Self {
        let mut result = Self::ONE;
        let mut base = self;
        let mut e = exp;
        while e > 0 {
            if e & 1 == 1 {
                result = result.mul(base);
            }
            base = base.mul(base);
            e >>= 1;
        }
        result
    }

    /// Multiplicative inverse (Fermat's little theorem with square-and-multiply).
    pub fn inv(self) -> Result<Self, GoldilocksError> {
        if self.0 == 0 {
            return Err(GoldilocksError::DivisionByZero);
        }
        // a^(P-2) mod P = a^(-1) mod P (Fermat's little theorem)
        // Use square-and-multiply with u128 arithmetic
        let base = self.0 as u128;
        let p = Self::P as u128;
        let mut result: u128 = 1;
        let mut b = base;
        let mut exp = Self::P - 2; // u64

        while exp > 0 {
            if exp & 1 == 1 {
                result = result.wrapping_mul(b) % p;
            }
            b = b.wrapping_mul(b) % p;
            exp >>= 1;
        }

        Ok(Self(result as u64))
    }

    /// Division.
    pub fn div(self, other: Self) -> Result<Self, GoldilocksError> {
        Ok(self.mul(other.inv()?))
    }

    /// Is zero?
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Debug for Goldilocks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GF({})", self.0)
    }
}

impl fmt::Display for Goldilocks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Add for Goldilocks {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.add(rhs)
    }
}

impl std::ops::Sub for Goldilocks {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.sub(rhs)
    }
}

impl std::ops::Mul for Goldilocks {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.mul(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let a = Goldilocks::new(100);
        let b = Goldilocks::new(200);
        assert_eq!(a.add(b), Goldilocks(300));
    }

    #[test]
    fn test_mul() {
        let a = Goldilocks::new(123);
        let b = Goldilocks::new(456);
        assert_eq!(a.mul(b), Goldilocks(123 * 456));
    }

    #[test]
    fn test_inv() {
        let a = Goldilocks::new(42);
        let a_inv = a.inv().unwrap();
        let product = a.mul(a_inv);
        eprintln!("a = {:?}", a);
        eprintln!("a_inv = {:?}", a_inv);
        eprintln!("product = {:?}", product);
        assert_eq!(product, Goldilocks::ONE);
    }

    #[test]
    fn test_sub() {
        let a = Goldilocks::new(100);
        let b = Goldilocks::new(200);
        let c = a.sub(b);
        // 100 - 200 mod p = p - 100
        assert_eq!(c, Goldilocks(Goldilocks::P - 100));
    }
}

}

//! # utqc-quantum
//!
//! Quantum algorithms — QFT, Grover, Shor, QPE.
//! Every algorithm compiles to the circuit IR.

use crate::core::{Circuit, Gate, Qubit, SingleGate, DoubleGate, CircuitError};
use crate::goldilocks::Goldilocks;
use thiserror::Error;

/// Quantum algorithm error.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum QuantumError {
    #[error("circuit error: {0}")]
    Circuit(#[from] CircuitError),
}

/// Quantum Fourier Transform on n qubits.
pub struct Qft;

impl Qft {
    /// Build a QFT circuit for `num_qubits` starting at qubit `start`.
    pub fn circuit(num_qubits: usize, start: usize) -> Result<Circuit, QuantumError> {
        let mut circ = Circuit::new(num_qubits, num_qubits);
        for i in 0..num_qubits {
            circ.add_gate(Gate::Single {
                gate: SingleGate::Hadamard,
                target: Qubit(start + i),
            })?;
            for j in (i + 1)..num_qubits {
                let angle = std::f64::consts::PI / (1u64 << (j - i)) as f64;
                circ.add_gate(Gate::Double {
                    gate: DoubleGate::CNOT,
                    control: Qubit(start + j),
                    target: Qubit(start + i),
                })?;
                circ.add_gate(Gate::Rotation {
                    target: Qubit(start + i),
                    angle,
                })?;
            }
        }
        // Swap qubits for standard ordering
        for i in 0..(num_qubits / 2) {
            circ.add_gate(Gate::Double {
                gate: DoubleGate::SWAP,
                control: Qubit(start + i),
                target: Qubit(start + num_qubits - 1 - i),
            })?;
        }
        Ok(circ)
    }
}

/// Grover's search algorithm.
pub struct Grover;

impl Grover {
    /// Build a Grover circuit for `num_qubits` with `num_solutions` marked states.
    pub fn circuit(num_qubits: usize, num_solutions: usize) -> Result<Circuit, QuantumError> {
        let mut circ = Circuit::new(num_qubits, num_qubits);

        // Initialize: Hadamard on all qubits
        for i in 0..num_qubits {
            circ.add_gate(Gate::Single {
                gate: SingleGate::Hadamard,
                target: Qubit(i),
            })?;
        }

        // Number of Grover iterations
        let iterations = Self::optimal_iterations(num_qubits, num_solutions);
        for _ in 0..iterations {
            // Oracle (mark solution states) — placeholder oracle
            Self::add_oracle(&mut circ, num_qubits)?;
            // Diffusion operator
            Self::add_diffusion(&mut circ, num_qubits)?;
        }

        // Measure all
        for i in 0..num_qubits {
            circ.add_measurement(Qubit(i), i)?;
        }

        Ok(circ)
    }

    /// Optimal number of Grover iterations.
    pub fn optimal_iterations(num_qubits: usize, num_solutions: usize) -> usize {
        let n = 1u64 << num_qubits;
        let m = num_solutions as f64;
        let ratio = n as f64 / m;
        let theta = (m / n as f64).sqrt().asin();
        let optimal = (std::f64::consts::FRAC_PI_4 / theta).round() as usize;
        optimal.max(1)
    }

    /// Add oracle gates (placeholder — marks first solution).
    fn add_oracle(circ: &mut Circuit, num_qubits: usize) -> Result<(), QuantumError> {
        // Placeholder: multi-controlled Z on all qubits
        if num_qubits >= 2 {
            circ.add_gate(Gate::Double {
                gate: DoubleGate::CZ,
                control: Qubit(0),
                target: Qubit(num_qubits - 1),
            })?;
        }
        Ok(())
    }

    /// Add diffusion operator.
    fn add_diffusion(circ: &mut Circuit, num_qubits: usize) -> Result<(), QuantumError> {
        // H on all, X on all, multi-controlled Z, X on all, H on all
        for i in 0..num_qubits {
            circ.add_gate(Gate::Single {
                gate: SingleGate::Hadamard,
                target: Qubit(i),
            })?;
            circ.add_gate(Gate::Single {
                gate: SingleGate::PauliX,
                target: Qubit(i),
            })?;
        }
        if num_qubits >= 2 {
            circ.add_gate(Gate::Double {
                gate: DoubleGate::CZ,
                control: Qubit(0),
                target: Qubit(num_qubits - 1),
            })?;
        }
        for i in 0..num_qubits {
            circ.add_gate(Gate::Single {
                gate: SingleGate::PauliX,
                target: Qubit(i),
            })?;
            circ.add_gate(Gate::Single {
                gate: SingleGate::Hadamard,
                target: Qubit(i),
            })?;
        }
        Ok(())
    }
}

/// Quantum Phase Estimation.
pub struct Qpe;

impl Qpe {
    /// Build a QPE circuit.
    pub fn circuit(num_counting_qubits: usize, target_qubit: usize) -> Result<Circuit, QuantumError> {
        let total = num_counting_qubits + 1;
        let mut circ = Circuit::new(total, num_counting_qubits);

        // Hadamard on counting qubits
        for i in 0..num_counting_qubits {
            circ.add_gate(Gate::Single {
                gate: SingleGate::Hadamard,
                target: Qubit(i),
            })?;
        }

        // Controlled unitary powers
        for i in 0..num_counting_qubits {
            let power = 1u64 << i;
            for _ in 0..power {
                circ.add_gate(Gate::Double {
                    gate: DoubleGate::CNOT,
                    control: Qubit(i),
                    target: Qubit(target_qubit),
                })?;
            }
        }

        // Inverse QFT on counting qubits
        let qft_circ = Qft::circuit(num_counting_qubits, 0)?;
        // Append gates in reverse (simplified — just add H gates)
        for i in 0..num_counting_qubits {
            circ.add_gate(Gate::Single {
                gate: SingleGate::Hadamard,
                target: Qubit(i),
            })?;
        }

        // Measure counting qubits
        for i in 0..num_counting_qubits {
            circ.add_measurement(Qubit(i), i)?;
        }

        Ok(circ)
    }
}

/// Shor's algorithm scaffold.
pub struct Shor;

impl Shor {
    /// Build a Shor circuit for factoring N (simplified).
    pub fn circuit(num_qubits: usize) -> Result<Circuit, QuantumError> {
        // Shor = QPE + modular exponentiation
        // Simplified: just build QPE on half the qubits
        let counting = num_qubits / 2;
        Qpe::circuit(counting, counting)
    }
}

/// Compile a circuit to Goldilocks field representation.
pub fn compile_to_goldilocks(circuit: &Circuit) -> Vec<Goldilocks> {
    circuit.gates.iter().map(|g| {
        match g {
            Gate::Single { gate, .. } => Goldilocks::new(*gate as u8 as u64),
            Gate::Double { gate, .. } => Goldilocks::new(*gate as u8 as u64 + 100),
            Gate::Rotation { angle, .. } => Goldilocks::new((*angle * 1000.0) as u64),
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qft() {
        let circ = Qft::circuit(3, 0).unwrap();
        assert!(circ.depth() > 0);
        assert!(circ.validate().is_ok());
    }

    #[test]
    fn test_grover() {
        let circ = Grover::circuit(3, 1).unwrap();
        assert!(circ.depth() > 0);
        assert!(circ.validate().is_ok());
    }

    #[test]
    fn test_qpe() {
        let circ = Qpe::circuit(2, 2).unwrap();
        assert!(circ.depth() > 0);
    }

    #[test]
    fn test_goldilocks_compile() {
        let circ = Qft::circuit(2, 0).unwrap();
        let field_elements = compile_to_goldilocks(&circ);
        assert!(!field_elements.is_empty());
    }
}

