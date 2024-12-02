#![feature(c_variadic)]
#![feature(core_intrinsics)]
#![allow(clippy::all)]
//! # KageShirei COFFEE Loader Module
//!
//! This module provides functionality for loading Beacon Object Files (BOFs) into memory
//! dynamically and managing their execution. It is designed to seamlessly integrate BOF loading
//! capabilities into the KageShirei agent, ensuring efficient resource handling and execution in
//! constrained environments.
pub mod loader;
