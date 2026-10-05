// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

//! # Embroidery Catalogue Core Library

pub mod config;
pub mod database;
pub mod error;
pub mod initial_setup;
pub mod logging;
pub mod models;
pub mod paths;
pub mod png_writer;
pub mod readers;
pub mod routes;
pub mod services;
pub mod settings;
pub mod state;
pub mod utils;

pub use state::*;
