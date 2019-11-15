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

use crate::pointer::{Pointer, Step};
use serde_value::Value;

/// A value pointer is what the traversal of the Serde pointer ends up at.
pub enum ValuePointer<'a> {
    /// The existing value found by the pointer.
    Existing(&'a Value),

    /// A pointer to a new element of the array found by the pointer.
    /// The first parameter is the parent array and the second parameter is the index
    /// to add the element to (i.e. the size of the array).
    NewUnder(&'a Value, usize),
}

pub(crate) fn traverse<'a>(val: &'a Value, pointer: &Pointer) -> Option<ValuePointer<'a>> {
    let mut it = pointer.clone().into_iter();
    match it.next() {
        Some(step) => _traverse(val, &step, &mut it),
        None => Some(ValuePointer::Existing(val)),
    }
}

fn _traverse<'a>(
    parent: &'a Value,
    step: &Step,
    steps: &mut dyn Iterator<Item = Step>,
) -> Option<ValuePointer<'a>> {
    let child = match step {
        Step::Name(name) => match parent {
            Value::Map(ref map) => map.get(&Value::String(name.clone())),
            _ => None,
        },
        Step::Index(index) => match parent {
            Value::Seq(ref seq) => seq.get(*index),
            _ => None,
        },
        Step::NewElement => None,
    };

    if child.is_none() {
        return match step {
            Step::NewElement => match parent {
                Value::Seq(ref seq) => match steps.next() {
                    // it is only possible to reference a new element by the last step
                    None => Some(ValuePointer::NewUnder(parent, seq.len())),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };
    }

    match steps.next() {
        Some(child_step) => _traverse(child.unwrap(), &child_step, steps),
        None => Some(ValuePointer::Existing(child.unwrap())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::str::FromStr;

    #[test]
    fn finds_all_data() {
        let val = Value::Map(BTreeMap::default());
        assert_eq!(&val, Pointer::from_str("#").unwrap().find(&val).unwrap());
    }

    #[test]
    fn finds_concrete_data() {
        let val = Value::Map(
            vec![
                (Value::String("k1".into()), Value::String("v1".into())),
                (
                    Value::String("k2".into()),
                    Value::Seq(vec![Value::I32(42), Value::Bool(true)]),
                ),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!(Value::Bool(true), *Pointer::from_str("/k2/1").unwrap().find(&val).unwrap());
    }

    #[test]
    fn find_new_element() {
        let val = Value::Map(
            vec![
                (Value::String("k1".into()), Value::String("v1".into())),
                (
                    Value::String("k2".into()),
                    Value::Seq(vec![Value::Bool(true)]),
                ),
            ]
            .into_iter()
            .collect(),
        );

        let found = Pointer::from_str("/k2/-").unwrap().traverse(&val).unwrap();
        match found {
            ValuePointer::NewUnder(parent, idx) => {
                assert_eq!(1, idx);
                match parent {
                    Value::Seq(ar) => {
                        assert_eq!(1, ar.len());
                    },
                    _ => panic!("Should have found an array of size 1")
                }
            },
            _ => panic!("Should have found value pointer to a new value.")
        }
    }
}
