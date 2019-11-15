/*
 *   Copyright (c) 2019 Lukas Krejci
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

use crate::parser;
use crate::traverse;
use serde_value::Value;
use std::str::FromStr;

/// Represents a pointer to the data as defined in the RFC6901 - the JSON Pointer specification.
#[derive(Eq, PartialEq, PartialOrd, Debug, Clone)]
pub struct Pointer {
    steps: Vec<Step>,
}

/// Represents a single traversal step of the pointer.
#[derive(Eq, PartialEq, PartialOrd, Debug, Clone)]
pub enum Step {
    Name(String),
    Index(usize),
    NewElement,
}

pub type ParseError = parser::ParseError;
pub type ValuePointer<'a> = traverse::ValuePointer<'a>;

impl Pointer {
    pub fn push(&mut self, step: Step) -> &mut Self {
        self.steps.push(step);
        self
    }

    pub fn pop(&mut self) -> Option<Step> {
        self.steps.pop()
    }

    pub fn insert(&mut self, index: usize, step: Step) {
        self.steps.insert(index, step)
    }

    pub fn remove(&mut self, index: usize) -> Step {
        self.steps.remove(index)
    }

    /// Traverses the provided value and finds the data this pointer points to in it, if any.
    pub fn traverse<'a>(&self, val: &'a Value) -> Option<ValuePointer<'a>> {
        traverse::traverse(val, self)
    }

    /// A simple override of `traverse()` that directly exposes the found value, if any.
    pub fn find<'a>(&self, val: &'a Value) -> Option<&'a Value> {
        match self.traverse(val) {
            Some(ValuePointer::Existing(v)) => Some(v),
            _ => None,
        }
    }
}

impl FromStr for Pointer {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::parse(s)
    }
}

impl From<Vec<Step>> for Pointer {
    fn from(ps: Vec<Step>) -> Self {
        Self { steps: ps }
    }
}

impl Into<Vec<Step>> for Pointer {
    fn into(self) -> Vec<Step> {
        self.steps
    }
}

impl IntoIterator for Pointer {
    type Item = Step;
    type IntoIter = std::vec::IntoIter<Step>;

    fn into_iter(self) -> Self::IntoIter {
        self.steps.into_iter()
    }
}

impl Default for Pointer {
    fn default() -> Self {
        Self {
            steps: Vec::default(),
        }
    }
}
