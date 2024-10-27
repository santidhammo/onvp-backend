/*
 *  ONVP Backend - Backend API provider for the ONVP website
 *
 * Copyright (c) 2024.  Sjoerd van Leent
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::api::middleware::authority::Allowance;
use crate::generic::http::Method;
use crate::model::primitives::{Role, RoleComposition};
use globset::{Glob, GlobMatcher};
use log::error;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct AuthorityConfig {
    map: HashMap<Method, Vec<(GlobMatcher, Allowance)>>,
}

impl AuthorityConfig {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn allow(self, method: Method, path: &str, allowance: Allowance) -> Self {
        let mut map = self.map;
        let pattern_result = Glob::new(path);
        match pattern_result {
            Err(e) => error!("Failed to compile glob pattern {}: {}", path, e),
            Ok(glob) => {
                let matcher = glob.compile_matcher();
                match map.get_mut(&method) {
                    None => {
                        map.insert(method, vec![(matcher, allowance)]);
                    }
                    Some(internal) => {
                        internal.push((matcher, allowance));
                    }
                }
            }
        }
        Self { map }
    }

    pub fn find(&self, method: Method, path: &str) -> Allowance {
        match self.map.get(&method) {
            None => Allowance::RoleAuthority(RoleComposition::from(Role::Operator)),
            Some(path_map) => {
                for (matcher, authorize) in path_map {
                    if matcher.is_match(path) {
                        return authorize.clone();
                    };
                }
                Allowance::RoleAuthority(RoleComposition::from(Role::Operator))
            }
        }
    }
}
